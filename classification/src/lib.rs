#[cfg(test)]
mod test {
    use std::{
        collections::HashMap, default, env, env::current_dir, fmt::Write, fs::read_to_string,
        ops::RangeInclusive,
    };
    use std::cmp::Ordering;
    use std::fs::File;
    use indicatif::{ProgressBar, ProgressIterator, ProgressState, ProgressStyle};
    use rust_bert::pipelines::{
        keywords_extraction::{Keyword, KeywordExtractionModel},
        ner::NERModel,
        question_answering::{QaInput, QuestionAnsweringModel},
        summarization::SummarizationModel,
    };
    use serde::Deserialize;
    use surrealdb::{Surreal, engine::local::RocksDb};
    use tokio::select;

    #[test]
    fn test_thing() {
        let keyword_extraction_model = KeywordExtractionModel::new(Default::default()).unwrap();
        let ner_model = NERModel::new(Default::default()).unwrap();
        let summarization_model = SummarizationModel::new(Default::default()).unwrap();
        let qa_model = QuestionAnsweringModel::new(Default::default()).unwrap();

        let input = "Rust is a multi-paradigm, general-purpose programming language. Rust \
                     emphasizes performance, type safety, and concurrency. Rust enforces memory \
                     safety—that is, that all references point to valid memory—without requiring \
                     the use of a garbage collector or reference counting present in other \
                     memory-safe languages. To simultaneously enforce memory safety and prevent \
                     concurrent data races, Rust's borrow checker tracks the object lifetime and \
                     variable scope of all references in a program during compilation. Rust is \
                     popular for systems programming but also offers high-level features \
                     including functional programming constructs.";
        let output = keyword_extraction_model.predict(&[input]).unwrap();

        println!("{output:?}");
        dbg!(current_dir());
        for i in 1..=3 {
            let input = read_to_string(dbg!(format!("src/keywords/test{i}.txt"))).unwrap();
            let input = [input];
            {
                let output = keyword_extraction_model.predict(&input).unwrap();
                println!("KeywordExtractionModel: {output:#?}");
            }
            {
                let output = ner_model.predict(&input);
                println!("NERModel: {output:#?}");
            }
            {
                let output = summarization_model.summarize(&input);
                println!("SummarizationModel: {output:#?}");
            }
            {
                let context = input[0].clone();
                let question = "What phrases or keywords can you pick out here?".into();
                let output = qa_model.predict(&[QaInput { question, context }], 1, 32);
                println!("QuestionAnsweringModel: {output:#?}");
            }
        }
    }

    #[test]
    fn test_combined() {
        println!("WD: {:?}", env::current_dir().unwrap());
        println!("loading data");
        let text = RangeInclusive::new(1, 3)
            .map(|i| read_to_string(format!("src/test{i}.txt")).unwrap())
            .collect::<Vec<_>>();
        println!("Loading model");
        let combined = {
            let keyword_extraction_model = KeywordExtractionModel::new(Default::default()).unwrap();
            keyword_extraction_model.predict(&text).unwrap()
        }
        .concat();

        let individual = {
            let keyword_extraction_model = KeywordExtractionModel::new(Default::default()).unwrap();
            text.iter()
                .map(|s| keyword_extraction_model.predict(&[s]).unwrap())
                .collect::<Vec<Vec<_>>>()
        }
        .concat()
        .concat();

        fn to_map(words: Vec<Keyword>) -> HashMap<String, f32> {
            words
                .into_iter()
                .map(|keyword| (keyword.text, keyword.score))
                .collect()
        }

        assert_eq!(to_map(combined), to_map(individual));
    }

    #[tokio::test]
    async fn test_massive() {
        let db = Surreal::new::<RocksDb>("../workshopdb").await.unwrap();

        #[derive(Deserialize)]
        struct Thing {
            title: String,
            description: String,
        }

        db.use_ns("workshop").use_db("workshop").await.unwrap();
        let mut resp = db
            .query(
                "SELECT * FROM workshop_items WHERE language=1 AND tags CONTAINSALL [tags:Mod, \
                 tags:⟨1.5⟩]  ORDER BY votes;",
            )
            .await
            .unwrap();
        dbg!(resp.take_errors());

        let items: Vec<Thing> = resp.take(0).unwrap();
        std::thread::spawn(move || {
            let keyword_extraction_model = KeywordExtractionModel::new(Default::default()).unwrap();
            println!("extracting {} items", items.len());
            let items = items
                .into_iter()
                .map(|pair| pair.description)
                .collect::<Vec<_>>()
                .to_vec();

            let chunk_size = 5;
            let pb = ProgressBar::new((items.len() / chunk_size) as u64);
            pb.set_style(
                ProgressStyle::with_template(
                    "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] \
                     {human_pos}/{human_len} ({eta})",
                )
                .unwrap()
                .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
                    write!(w, "{}", humantime::format_duration(state.eta()).to_string()).unwrap()
                })
                .progress_chars("#>-"),
            );
            println!("remapped");
            let keywords = items
                .chunks(chunk_size)
                .progress_with(pb)
                .map(|set| keyword_extraction_model.predict(set).unwrap().concat())
                .map(|words| {
                    words
                        .into_iter()
                        .map(|word| (word.text, word.score))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
                .concat();
            println!("{}", keywords.len());
            let mut map = HashMap::new();
            for (keyword, score) in keywords {
                map.entry(keyword)
                    .and_modify(|vec: &mut Vec<_>| vec.push(score))
                    .or_insert_with(|| vec![score]);
            }
            let mut summed = map.into_iter()
                .map(|(key, vals)| (key, vals.iter().sum::<f32>() / vals.len() as f32))
                .collect::<Vec<_>>();
            summed.sort_unstable_by(|a,b|a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));

            let mut file = File::create("/tmp/foo.txt").unwrap();
            for (word, score) in &summed {
                use std::io::Write;
                writeln!(&mut file, "{word}:{score}").unwrap();
            }

            println!("{summed:#?}");
        })
        .join()
        .unwrap();
    }
}
