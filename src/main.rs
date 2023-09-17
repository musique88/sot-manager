use rhai::{Engine, Func, EvalAltResult, Dynamic, Scope};
use sqlx::postgres::PgPoolOptions;
use std::{sync::Arc, error::Error};
use rhai::Array;
use serde_json::json;

fn ssh_connect(ip: &str, username: &str, password: &str, commands: Vec<&str>) -> String {
    "".to_string()
}

fn get(url: &str) -> Result<Array, Box<EvalAltResult>> {
    match reqwest::blocking::get(url) {
        Ok(res) => {
            let status = res.status().as_str().to_string();
            match res.text() {
                Ok(string) => {
                    Ok(vec![status.into(), string.into()])
                }
                Err(err) => {
                    Ok(vec![status.into(), err.to_string().into()])
                }
            }
        }
        Err(err) => {
            Err(err.to_string().into())
        }
    } 
}

fn post_text(url: &str, text: &str) -> Result<Array, Box<EvalAltResult>> {
    match reqwest::blocking::Client::new().post(url).body(String::from(text)).send() {
        Ok(res) => {
            let status = res.status().as_str().to_string();
            match res.text() {
                Ok(string) => {
                    Ok(vec![status.into(), string.into()])
                }
                Err(err) => {
                    Ok(vec![status.into(), err.to_string().into()])
                }
            }
        }
        Err(err) => {
            Err(err.to_string().into())
        }
    }
}

fn post_json(url: &str, json: rhai::Map) -> Result<Array, Box<EvalAltResult>> {
    match reqwest::blocking::Client::new()
        .post(url)
        .body(serde_json::to_value(json).unwrap().to_string())
        .send() {
        Ok(res) => {
            let status = res.status().as_str().to_string();
            match res.text() {
                Ok(string) => {
                    Ok(vec![status.into(), string.into()])
                }
                Err(err) => {
                    Ok(vec![status.into(), err.to_string().into()])
                }
            }
        }
        Err(err) => {
            Err(err.to_string().into())
        }
    }
}

type ComparableInfo = rhai::Map;

trait Queryable {
    fn query(&mut self, info: rhai::Map) -> ComparableInfo;
    fn get_last_info(&self) -> ComparableInfo;
}

struct Endpoint {
    name: String,
    queryables: Vec<Box<dyn Queryable>>,
    last_info: ComparableInfo
}

impl Endpoint {
    fn new(name: String) -> Endpoint {
        Endpoint { name: name.to_string(), queryables: vec![], last_info: rhai::Map::new()}
    }
}

impl Queryable for Endpoint {
    fn query(&mut self, json: rhai::Map) -> ComparableInfo {
        self.last_info = rhai::Map::new();
        self.last_info.clone()
    }
    fn get_last_info(&self) -> ComparableInfo {
        self.last_info.clone()
    }
}

struct Manager {
    
}

struct Script {
    name: String,
    script: String,
}

const FUNCTION_NAME: &str = "run";

impl Script {
    fn new(name: String, script: String) -> Result<Script, Box<dyn Error>> {
        let engine = Engine::new();
        match engine.compile(script.clone()) {
            Ok(ast) => {
                let mut has_func = false;
                for f in ast.iter_functions() {
                    if f.name.eq(FUNCTION_NAME) && f.params.len() == 1 {
                        has_func = true;
                    }
                };
                if has_func {
                    Ok(Script {script: script.clone(), name: name.clone()})
                } else {
                    Err("'run' function was not found or not enough parameters".into())
                }
            },
            Err(error) => {
                Err(error.to_string().into())
            }

        }
    }
}

impl Queryable for Script {
    fn query(&mut self, info: rhai::Map) -> ComparableInfo {
        let mut engine = Engine::new();
        engine.register_fn("get", get);
        engine.register_fn("post_json", post_json);
        engine.register_fn("post_text", post_text);
        match engine.compile(self.script.clone()) {
            Ok(ast) => {
                let mut scope = Scope::new();
                engine.call_fn::<rhai::Map>(&mut scope, &ast, "run", (info,)).unwrap()
            },
            Err(err) => {
                let mut map = rhai::Map::new();
                map.insert("error".into(), err.to_string().into());
                map
            }
        }
    }

    fn get_last_info(&self) -> ComparableInfo {
        ComparableInfo::new()
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    //let pool = PgPoolOptions::new()
    //    .max_connections(5)
    //    .connect("postgres://admin:@localhost/db").await?;

    //let row: (i64,) = sqlx::query_as("SELECT $1")
    //    .bind(150_i64)
    //    .fetch_one(&pool).await?;

    let test_script = "\
        fn run(run_information) { \
            print(get(run_information.ip)); 
            #{test: 23}
        }\
        ";

    tokio::task::spawn_blocking(move | | -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut script = Script::new("aaa".to_string(), test_script.to_string()).unwrap();
        let mut map = rhai::Map::new();
        map.insert("ip".into(), "http://google.com".into());
        map.insert("whatever".into(), "a".into());
        println!("{:?}", script.query(map));
        Ok(())
    });
    Ok(())
}


