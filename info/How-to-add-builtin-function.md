# how to add a builtin function

### Declare function

* Declare function name in `pub enum Function ` of [src/builtins.rs](../src/builtins.rs)
* Bind function name in `static_map!` of [src/builtins.rs](../src/builtins.rs)
* Add function name in `arity`(参数个数) function of [src/builtins.rs](../src/builtins.rs)
* Add function name in `type_sig`(函数类型签名) of [src/builtins.rs](../src/builtins.rs)
* Add function name in `step`(函数返回值类型) of [src/builtins.rs](../src/builtins.rs)

### Bound with AWK compiler

* add display for `impl Display for Function`(字符串化) of [src/display.rs](../src/display.rs)
* Add function name in `builtin`(指令集) of [src/compile.rs](../src/compile.rs)
* Add function name in `accum`(累加器) of [src/bytecode.rs](../src/bytecode.rs)
* Add function name in `visit_ll`(遍历器) of [src/dataflow.rs](../src/dataflow.rs)
* Add function name in `run_at`() of [src/interp.rs](../src/interp.rs) 重点看一下这个函数
* register function in `register! {` of [src/codegen/intrinsics.rs](../src/codegen/intrinsics.rs)

### Code implementation

* Add function name in `gen_ll_inst` of [src/codegen/mod.rs](../src/codegen/mod.rs)
* Add function implementation: 

```
pub(crate) unsafe extern "C" fn uuid(runtime: *mut c_void) -> U128 {
    let res = Str::from("demo");
    mem::transmute::<Str, U128>(res)
}
```