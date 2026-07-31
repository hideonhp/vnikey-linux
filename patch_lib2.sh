sed -i 's/assert_eq!(engine.buffer.len(), 16);/assert_eq!(engine.raw_buffer.len(), 16);/' src/lib.rs
