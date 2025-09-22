use pyo3::prelude::*;
//use reqwest::blocking::Client;
//use serde::{Deserialize, Serialize};
use std::env;
use dotenv::dotenv;
use pyo3::exceptions::PyValueError;
//use crate::error;

#[pyfunction]
pub fn get_youtube_api_key() -> PyResult<String> {

    dotenv().ok();
    match env::var("YOUTUBE_API_KEY") {
        Ok(key) => Ok(key),
        Err(_) => Err(PyValueError::new_err("You must set the environment variable YOUTUBE_API_KEY"))
    }
}


#[pyfunction]
pub fn call_youtube_client(endpoint_url: Option<String>, api_key: Option<String>) -> PyResult<()> {

    println!("endpoint_url: {:?}", endpoint_url);
    println!("api_key: {:?}", api_key);
    Ok(())
}
