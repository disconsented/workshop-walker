This module requires rust-bert to be cloned locally, and, have the following patch applied (probably).

```diff
Index: Cargo.toml
IDEA additional info:
Subsystem: com.intellij.openapi.diff.impl.patch.CharsetEP
<+>UTF-8
===================================================================
diff --git a/Cargo.toml b/Cargo.toml
--- a/Cargo.toml	(revision 411c224cfc427f1b7b7fb6c1e449505427ba3d92)
+++ b/Cargo.toml	(date 1746583330894)
@@ -76,7 +76,7 @@
 
 [dependencies]
 rust_tokenizers = "8.1.1"
-tch = { version = "0.17.0", features = ["download-libtorch"] }
+tch = { version = "0.20.0", features = ["download-libtorch"] }
 serde_json = "1"
 serde = { version = "1", features = ["derive"] }
 ordered-float = "4.2.0"
```