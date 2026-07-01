use crate::app::config::ReactorConfig;
use crate::app::context::ReactorContext;
use crate::app::runner::run;
use crate::app::ReactorApp;

struct QuickApp<I, U>
where
    I: FnMut(&mut ReactorContext) + 'static,
    U: FnMut(&mut ReactorContext) + 'static,
{
    config: ReactorConfig,
    init: Option<I>,
    update: U,
}

impl<I, U> ReactorApp for QuickApp<I, U>
where
    I: FnMut(&mut ReactorContext) + 'static,
    U: FnMut(&mut ReactorContext) + 'static,
{
    fn config(&self) -> ReactorConfig { self.config.clone() }
    fn init(&mut self, ctx: &mut ReactorContext) { if let Some(mut f) = self.init.take() { f(ctx); } }
    fn update(&mut self, ctx: &mut ReactorContext) { (self.update)(ctx); }
}

pub fn quick<U>(title: &str, width: u32, height: u32, update: U)
where U: FnMut(&mut ReactorContext) + 'static {
    run(QuickApp {
        config: ReactorConfig::new(title).with_size(width, height),
        init: None::<fn(&mut ReactorContext)>,
        update,
    });
}

pub fn quick_with<I, U>(config: ReactorConfig, init: I, update: U)
where I: FnMut(&mut ReactorContext) + 'static, U: FnMut(&mut ReactorContext) + 'static {
    run(QuickApp { config, init: Some(init), update });
}

#[inline(always)]
pub fn call_init<F>(mut f: F, ctx: &mut ReactorContext) where F: FnMut(&mut ReactorContext) { f(ctx); }

#[inline(always)]
pub fn call_update<F>(mut f: F, ctx: &mut ReactorContext) where F: FnMut(&mut ReactorContext) { f(ctx); }
