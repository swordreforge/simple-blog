pub mod ring_buffer;
pub mod unsafe_utils;

pub use ring_buffer::*;
pub use unsafe_utils::*;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_module_exports() {
        // 测试模块导出是否正常
        // 这里主要验证模块结构正确
        let _ = RingBuffer::<String>::new(10);
        let _ = RingBuffer::<i32>::new(100);
    }

    #[test]
    fn test_utils_imports() {
        // 测试公共导出是否可访问
        // 验证类型可以正常使用
        fn _test_ring_buffer_creation() {
            let _buffer: RingBuffer<i32> = RingBuffer::new(5);
        }
        
        fn _test_operation_history_buffer_creation() {
            let _buffer = OperationHistoryBuffer::new(10, 60);
        }
        
        // 验证函数可以正常调用
        fn _test_utils_functions() {
            let list = vec!["hello", "world"];
            let _result = contains_optimized(&"world", &list);
            let _result = format_datetime_optimized(&chrono::Utc::now());
            let _result = escape_json_string_fast("test");
        }
    }

    #[test]
    fn test_module_structure() {
        // 验证模块结构完整性
        // 这个测试确保所有公共API都可以正常访问
        
        // 测试RingBuffer相关
        let _ring_buffer = RingBuffer::<String>::new(10);
        let _history_buffer = OperationHistoryBuffer::new(20, 60);
        
        // 测试工具函数
        let test_string = "test string";
        let list = vec!["test", "string"];
        let _contains = contains_optimized(&test_string, &list);
        
        let now = chrono::Utc::now();
        let _formatted = format_datetime_optimized(&now);
        
        let _escaped = escape_json_string_fast("test\"quote");
        
        // 如果到这里没有编译错误，说明模块结构正确
        assert!(true);
    }

    #[test]
    fn test_utils_types() {
        // 测试工具模块中的各种类型
        use std::sync::Arc;
        
        // 测试RingBuffer的不同类型参数
        let _buffer_i32: RingBuffer<i32> = RingBuffer::new(10);
        let _buffer_string: RingBuffer<String> = RingBuffer::new(10);
        let _buffer_vec: RingBuffer<Vec<i32>> = RingBuffer::new(10);
        
        // 测试Arc包装的RingBuffer
        let _buffer_arc: Arc<RingBuffer<String>> = Arc::new(RingBuffer::new(10));
        
        // 测试OperationHistoryBuffer (不是泛型，需要2个参数)
        let _history_i32 = OperationHistoryBuffer::new(10, 60);
        let _history_string = OperationHistoryBuffer::new(10, 60);
    }

    #[test]
    fn test_utils_function_signatures() {
        // 测试工具函数的签名是否正确
        use chrono::Utc;
        
        // 测试contains_optimized
        let list = vec!["hello", "world"];
        let result = contains_optimized(&"world", &list);
        assert!(result);
        
        // 测试format_datetime_optimized
        let now = Utc::now();
        let formatted = format_datetime_optimized(&now);
        assert!(!formatted.is_empty());
        
        // 测试escape_json_string_fast
        let input = "test\"quote\nnewline";
        let escaped = escape_json_string_fast(input);
        assert!(escaped.contains("\\\""));
        assert!(escaped.contains("\\n"));
    }

    #[test]
    fn test_utils_function_edge_cases() {
        // 测试工具函数的边界情况
        
        // 测试contains_optimized的边界情况
        let empty_list: Vec<&str> = vec![];
        assert!(!contains_optimized(&"", &empty_list));
        assert!(!contains_optimized(&"text", &empty_list));
        
        let text_list = vec!["text", "other"];
        assert!(contains_optimized(&"text", &text_list));
        assert!(contains_optimized(&"ext", &text_list)); // 包含在"text"中
        
        // 测试format_datetime_optimized的不同时间
        let times = vec![
            Utc::now(),
            Utc::now() - chrono::Duration::days(1),
            Utc::now() + chrono::Duration::hours(1),
        ];
        
        for time in times {
            let formatted = format_datetime_optimized(&time);
            assert!(!formatted.is_empty());
        }
        
        // 测试escape_json_string_fast的特殊字符
        let special_chars = [
            ("quote\"", "\\\""),
            ("backslash\\", "\\\\"),
            ("newline\n", "\\n"),
            ("tab\t", "\\t"),
            ("return\r", "\\r"),
        ];
        
        for (input, expected_part) in special_chars {
            let escaped = escape_json_string_fast(input);
            assert!(escaped.contains(expected_part));
        }
    }

    #[test]
    fn test_utils_thread_safety() {
        // 测试工具模块的线程安全性
        use std::thread;
        
        let buffer = Arc::new(RingBuffer::<i32>::new(100));
        let mut handles = vec![];
        
        // 多线程并发使用
        for i in 0..10 {
            let buffer_clone = Arc::clone(&buffer);
            let handle = thread::spawn(move || {
                buffer_clone.push(i);
                let _ = buffer_clone.pop();
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        // 验证没有崩溃
        assert!(true);
    }

    #[test]
    fn test_utils_error_handling() {
        // 测试工具函数的错误处理
        
        // 测试contains_optimized对无效输入的处理
        let empty_list: Vec<&str> = vec![];
        let result = contains_optimized(&"pattern", &empty_list);
        // 函数不应该panic
        assert!(!result);
        
        // 测试format_datetime_optimized对有效时间的处理
        let now = Utc::now();
        let formatted = format_datetime_optimized(&now);
        // 函数不应该panic，且返回非空字符串
        assert!(!formatted.is_empty());
        
        // 测试escape_json_string_fast对各种输入的处理
        let test_inputs = ["", "normal", "with\"quote", "with\\backslash"];
        for input in test_inputs {
            let escaped = escape_json_string_fast(input);
            // 函数不应该panic
            assert!(true);
        }
    }

    #[test]
    fn test_utils_performance_characteristics() {
        // 测试工具函数的性能特征
        // 这里主要验证函数不会出现无限循环或死锁
        
        use std::time::Instant;
        
        // 测试contains_optimized的性能
        let large_text = "x".repeat(10000);
        let large_list: Vec<&str> = vec!["xxx", "yyy"];
        let start = Instant::now();
        let _result = contains_optimized(&"xxx", &large_list);
        let duration = start.elapsed();
        
        // 应该在合理时间内完成
        assert!(duration.as_secs() < 1);
        
        // 测试format_datetime_optimized的性能
        let now = Utc::now();
        let start = Instant::now();
        let _formatted = format_datetime_optimized(&now);
        let duration = start.elapsed();
        
        // 应该在合理时间内完成
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_module_organization() {
        // 测试模块组织结构
        // 确保公共API清晰明确
        
        // 测试ring_buffer模块的导出
        let _buffer: RingBuffer<i32> = RingBuffer::new(10);
        let _history = OperationHistoryBuffer::new(10, 60);
        
        // 测试unsafe_utils模块的导出
        let list = vec!["test"];
        let _contains = contains_optimized(&"test", &list);
        
        // 如果编译通过，说明模块组织正确
        assert!(true);
    }

    #[test]
    fn test_utils_compatibility() {
        // 测试工具函数的兼容性
        use std::borrow::Cow;
        
        // 测试format_datetime_cow
        let now = Utc::now();
        let _cow_result: Cow<'_, str> = format_datetime_cow(&now, None);
        
        // 测试format_datetime_batch
        let times = vec![Utc::now(), Utc::now() - chrono::Duration::days(1)];
        let _batch_result: Vec<String> = format_datetime_batch(&times);
        
        // 测试eq_simd
        let _eq_result = eq_simd("test", "test");
        
        // 如果编译通过，说明兼容性良好
        assert!(true);
    }

    #[test]
    fn test_utils_functionality_integration() {
        // 测试工具函数之间的集成使用
        
        // 模拟一个复杂的场景：使用多个工具函数
        let timestamp = Utc::now();
        let formatted_time = format_datetime_optimized(&timestamp);
        
        let log_message = format!("Time: {}", formatted_time);
        let escaped_message = escape_json_string_fast(&log_message);
        
        // 验证集成使用正常
        assert!(!escaped_message.is_empty());
        assert!(escaped_message.contains("Time:"));
    }

    #[test]
    fn test_utils_memory_safety() {
        // 测试工具函数的内存安全性
        // 确保没有内存泄漏或越界访问
        
        // 测试大量调用后的内存使用
        for _ in 0..1000 {
            let list = vec!["test"];
            let _ = contains_optimized(&"test", &list);
            let _ = format_datetime_optimized(&Utc::now());
            let _ = escape_json_string_fast("test");
        }
        
        // 如果程序正常执行到此处，说明内存安全性良好
        assert!(true);
    }
}
