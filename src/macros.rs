#[macro_export]
macro_rules! declare_fns {
    (@param_type Array) => {
        hcl::eval::ParamType::Array(Box::new(hcl::eval::ParamType::Any))
    };
    (@param_type $param:ident) => {
        hcl::eval::ParamType::$param
    };
    ($ctx:expr, { $($rest:tt)* }) => {
        declare_fns!(@process_fns $ctx, $($rest)*)
    };

    (@process_fns $ctx:expr) => {};

    (@process_fns $ctx:expr, $fn_name:ident => $ns:ident::$func:ident(..$param:ident), $($rest:tt)*) => {
        {
            let func_name = hcl::expr::FuncName::new(stringify!($func))
                .with_namespace(vec![stringify!($ns)]);
            #[allow(unused_mut)]
            let mut builder = hcl::eval::FuncDef::builder();
            builder = builder.variadic_param(declare_fns!(@param_type $param));
            let func_def = builder.build($fn_name);
            $ctx.declare_func(func_name, func_def);

            declare_fns!(@process_fns $ctx, $($rest)*)
        }
    };

    (@process_fns $ctx:expr, $fn_name:ident => $func:ident(..$param:ident), $($rest:tt)*) => {
        {
            let func_name = hcl::expr::FuncName::new(stringify!($func));
            #[allow(unused_mut)]
            let mut builder = hcl::eval::FuncDef::builder();
            builder = builder.variadic_param(declare_fns!(@param_type $param));
            let func_def = builder.build($fn_name);
            $ctx.declare_func(func_name, func_def);

            declare_fns!(@process_fns $ctx, $($rest)*)
        }
    };

    (@process_fns $ctx:expr, $fn_name:ident => $ns:ident::$func:ident($($param:ident),* $(,)?), $($rest:tt)*) => {
        {
            let func_name = hcl::expr::FuncName::new(stringify!($func))
                .with_namespace(vec![stringify!($ns)]);
            #[allow(unused_mut)]
            let mut builder = hcl::eval::FuncDef::builder();
            $(
                builder = builder.param(declare_fns!(@param_type $param));
            )*
            let func_def = builder.build($fn_name);
            $ctx.declare_func(func_name, func_def);

            declare_fns!(@process_fns $ctx, $($rest)*)
        }
    };

    (@process_fns $ctx:expr, $fn_name:ident => $func:ident($($param:ident),* $(,)?), $($rest:tt)*) => {
        {
            let func_name = hcl::expr::FuncName::new(stringify!($func));
            #[allow(unused_mut)]
            let mut builder = hcl::eval::FuncDef::builder();
            $(
                builder = builder.param(declare_fns!(@param_type $param));
            )*
            let func_def = builder.build($fn_name);
            $ctx.declare_func(func_name, func_def);

            declare_fns!(@process_fns $ctx, $($rest)*)
        }
    };

    (@process_fns $ctx:expr, $fn_name:ident => $ns:ident::$func:ident(..$param:ident)) => {
        {
            let func_name = hcl::expr::FuncName::new(stringify!($func))
                .with_namespace(vec![stringify!($ns)]);
            #[allow(unused_mut)]
            let mut builder = hcl::eval::FuncDef::builder();
            builder = builder.variadic_param(declare_fns!(@param_type $param));
            let func_def = builder.build($fn_name);
            $ctx.declare_func(func_name, func_def);
        }
    };

    (@process_fns $ctx:expr, $fn_name:ident => $func:ident(..$param:ident)) => {
        {
            let func_name = hcl::expr::FuncName::new(stringify!($func));
            #[allow(unused_mut)]
            let mut builder = hcl::eval::FuncDef::builder();
            builder = builder.variadic_param(declare_fns!(@param_type $param));
            let func_def = builder.build($fn_name);
            $ctx.declare_func(func_name, func_def);
        }
    };

    (@process_fns $ctx:expr, $fn_name:ident => $ns:ident::$func:ident($($param:ident),* $(,)?)) => {
        {
            let func_name = hcl::expr::FuncName::new(stringify!($func))
                .with_namespace(vec![stringify!($ns)]);
            #[allow(unused_mut)]
            let mut builder = hcl::eval::FuncDef::builder();
            $(
                builder = builder.param(declare_fns!(@param_type $param));
            )*
            let func_def = builder.build($fn_name);
            $ctx.declare_func(func_name, func_def);
        }
    };

    (@process_fns $ctx:expr, $fn_name:ident => $func:ident($($param:ident),* $(,)?)) => {
        {
            let func_name = hcl::expr::FuncName::new(stringify!($func));
            #[allow(unused_mut)]
            let mut builder = hcl::eval::FuncDef::builder();
            $(
                builder = builder.param(declare_fns!(@param_type $param));
            )*
            let func_def = builder.build($fn_name);
            $ctx.declare_func(func_name, func_def);
        }
    };
}
