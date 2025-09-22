use pyo3::prelude::*;

pub mod auth;
pub mod account;

#[pymodule]
fn youtube_stats(py: Python, m: &PyModule) -> PyResult<()> {
    let auth_module = PyModule::new(py, "auth")?;

    auth_module.add_function(wrap_pyfunction!(auth::get_youtube_api_key, auth_module)?)?;
    auth_module.add_function(wrap_pyfunction!(auth::call_youtube_client, auth_module)?)?;

    let account_module = PyModule::new(py, "account")?;
    account_module.add_function(wrap_pyfunction!(account::get_youtube_channel_stats, account_module)?)?;
    account_module.add_function(wrap_pyfunction!(account::search_youtube_channels, account_module)?)?;

    m.add_submodule(auth_module)?;
    m.add_submodule(account_module)?;

    py.import("sys")?.getattr("modules")?.set_item("youtube_stats.auth", auth_module)?;
    py.import("sys")?.getattr("modules")?.set_item("youtube_stats.account", account_module)?;
    Ok(())

}
