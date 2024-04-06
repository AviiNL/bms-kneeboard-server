use std::collections::HashMap;

use once_cell::sync::Lazy;
use tera::{to_value, Context, Result as TeraResult, Tera, Value};

macro_rules! add_template {
    ($tera:expr, $x:literal) => {
        $tera
            .add_raw_template(
                concat!($x, ".html.twig"),
                include_str!(concat!("../templates/", $x, ".html.twig")),
            )
            .unwrap();
    };
}

static TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    add_template!(tera, "index");
    add_template!(tera, "overview");
    add_template!(tera, "sitrep");
    add_template!(tera, "roster");
    add_template!(tera, "elements");
    add_template!(tera, "threatanalysis");
    add_template!(tera, "steerpoints");
    add_template!(tera, "commladder");
    add_template!(tera, "iff");
    add_template!(tera, "ordnance");
    add_template!(tera, "weather");
    add_template!(tera, "support");
    add_template!(tera, "roe");
    add_template!(tera, "emergency");

    tera.register_filter("nl2br", nl2br);

    tera
});

fn nl2br(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let Some(value) = value.as_str() else {
        return Ok(value.clone());
    };

    Ok(to_value(value.replace('\n', "<br/>")).unwrap())
}

pub fn render(context: Context) -> Result<String, Box<dyn std::error::Error>> {
    Ok(TERA.render("index.html.twig", &context)?)
}
