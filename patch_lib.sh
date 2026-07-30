cat << 'PATCH_EOF' > patch.diff
--- src/lib.rs
+++ src/lib.rs
@@ -115,4 +115,35 @@
         assert_eq!(engine.state, State::Composing);
         assert_eq!(engine.raw_buffer.as_slice(), ['b']);
     }
+
+    #[test]
+    fn test_smart_english() {
+        let mut engine = Engine::new();
+        let word = "english";
+        for (i, c) in word.chars().enumerate() {
+            let action = engine.process_key(c);
+            let expected = &word[..i+1];
+            assert_eq!(action, Action::Preedit(make_buffer(expected)), "Failed at char: {}", c);
+        }
+    }
+
+    #[test]
+    fn test_smart_linux() {
+        let mut engine = Engine::new();
+        let word = "linux";
+        for (i, c) in word.chars().enumerate() {
+            let action = engine.process_key(c);
+            let expected = &word[..i+1];
+            assert_eq!(action, Action::Preedit(make_buffer(expected)), "Failed at char: {}", c);
+        }
+    }
+
+    #[test]
+    fn test_valid_telex() {
+        let mut engine = Engine::new();
+        engine.process_key('h');
+        engine.process_key('o');
+        engine.process_key('a');
+        let action1 = engine.process_key('s'); // a+s -> á. hoas -> hoá
+        assert_eq!(action1, Action::Preedit(make_buffer("hoá")));
+    }
 }
PATCH_EOF
patch -p0 < patch.diff
