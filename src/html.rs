use once_cell::sync::Lazy;
use tera::{Context, Tera};

static TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.add_raw_template(
        "index.html.twig",
        include_str!("../templates/index.html.twig"),
    )
    .unwrap();
    tera.add_raw_template(
        "elements.html.twig",
        include_str!("../templates/elements.html.twig"),
    )
    .unwrap();
    tera.add_raw_template(
        "steerpoints.html.twig",
        include_str!("../templates/steerpoints.html.twig"),
    )
    .unwrap();
    tera.add_raw_template(
        "commladder.html.twig",
        include_str!("../templates/commladder.html.twig"),
    )
    .unwrap();

    // tera.register_filter("iconize", iconize);
    // tera.register_filter("from_ico", from_ico);
    // tera.register_filter("size", size);
    // tera.register_filter("time", time);
    // tera.register_filter("md", md);

    tera
});

pub struct RenderOptions {
    pub show_elements: bool,
    pub eneble_steerpoints: bool,
    pub show_comm_ladder: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            show_elements: true,
            eneble_steerpoints: true,
            show_comm_ladder: true,
        }
    }
}

impl RenderOptions {
    pub fn render(&self, context: Context) -> Result<String, Box<dyn std::error::Error>> {
        Ok(TERA.render("index.html.twig", &context)?)
    }
}
