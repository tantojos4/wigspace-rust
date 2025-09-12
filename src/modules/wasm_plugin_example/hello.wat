(module
  (type $handle_t (func (param i32 i32) (result i32)))
  (import "env" "memory" (memory 1))
  (func $handle (export "handle") (param $ptr i32) (param $len i32) (result i32)
    ;; For demo: always return pointer to static string
    (i32.const 0)
  )
  (data (i32.const 0) "[wasm_plugin] hello wasm\00")
)
