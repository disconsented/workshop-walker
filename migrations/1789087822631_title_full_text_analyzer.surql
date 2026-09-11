-- Stupid surrealdb bug that never got fixed https://github.com/surrealdb/surrealdb/issues/7234
DEFINE ANALYZER title_analyzer TOKENIZERS punct, blank FILTERS ascii, lowercase;