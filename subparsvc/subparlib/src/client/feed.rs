use std::fmt;

#[derive(Clone)]
pub struct Feed {
    url: hyper::Uri,
    label: &'static str,
}

impl Feed {
    pub fn new(label: &'static str, url: &str) -> Self {
        Feed {
            label,
            url: url.parse().expect("bad url"),
        }
    }
    pub fn url(&self) -> &hyper::Uri {
        &self.url
    }
    pub fn name(&self) -> &'static str {
        self.label
    }
    pub fn from_static(f: &'static str) -> Self {
        let base = "https://api-endpoint.mta.info/Dataservice/mtagtfsfeeds/nyct%2Fgtfs";
        let url = match f {
            "1234567" => base.to_string(),
            | "ace"
            | "bdfm"
            | "g"
            | "jz"
            | "nqrw"
            | "l"
            | "si"
            => format!("{}-{}", base, f),
            _ => panic!("unrecognized feed {f}"),
        };
        Feed::new(f, &url)
    }
}

impl fmt::Display for Feed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "F.{:<7}", self.label)
    }
}
impl fmt::Debug for Feed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "F.{:<7}", self.label)
    }
}
