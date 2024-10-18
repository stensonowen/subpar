use subpar::{api};

fn score_borough(b: &str) -> u32 {
    match b {
        "M"  | "Manhattan" => 0,
        "Bk" | "Brooklyn" => 1,
        "Q"  | "Queens" => 2,
        "Bx" | "Bronx" => 3,
        "SI" | "Staten Island" => 4,
        _ => panic!("unknown borough '{b}'"),
    }
}

fn spell_borough(b: &str) -> &'static str {
    match b {
        "M"  => "Manhattan",
        "Bk" => "Brooklyn",
        "Q"  => "Queens",
        "Bx" => "Bronx",
        "SI" => "Staten Island",
        _ => panic!("unknown borough '{b}'"),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = api::Client::default();
    let mut complexes = client.get_complexes().await?;
    complexes.sort_by_key(|c| {
        (score_borough(&c.borough), 100 - c.routes.len())
    });
    for cplx in &complexes {
        let mut rs = cplx.routes.clone();
        rs.sort();
        let rs = rs.iter()
            .map(|r| format!(r#"<img class="icon bullet" alt="{r}" src="/f/R{r}.svg" />"#))
            .fold(String::new(), |a, b| a + " " + &b);
        let id = cplx.complex_id;
        let ada = {
            let (col, txt) = match cplx.ada {
                api::AdaStatus::Full =>     ("G", "Full"),
                api::AdaStatus::Partial =>  ("Y", "Part"),
                api::AdaStatus::No =>       ("R", "None"),
            };
            format!(r#"<img id="ada{id}" class="icon" alt="{txt}" src="/f/ada{col}.svg"> {txt} </img>"#)
        };
        let borough = spell_borough(&cplx.borough);
        let name = &cplx.stop_name;
        let link = format!(r#"<a href="/c/{id}"> {name} </a>"#);
        println!(r#"<tr> <td>{rs}</td> <td>{ada}</td> <td>{link}</td> <td>{borough}</td> </tr>"#);
    }
        

    Ok(())
}

