use clap::App;
use clap::Arg;
use clap::AppSettings;
use clap::ArgMatches;
use clap::SubCommand;

use nirah_core::core::NirahResult;
use nirah_core::rpc::RpcRequest;
use nirah_core::rpc::RpcResponse;
use nirah_core::config::VariableValue;
use nirah_core::config::VariableKey;
use ascii_table::print_table;
use ascii_table::ColumnConfig;
use ascii_table::TableConfig;
use ascii_table::Align;

use super::utils::print_response;
use super::utils::get_response;
use super::utils::OptionalDisplay;

pub fn args() -> App<'static, 'static> {
    SubCommand::with_name("config")
        .about("Manage config variables")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("list")
            .about("Show all set configuration variables")
        )
        .subcommand(SubCommand::with_name("get")
            .about("Get value of config variable.")
            .setting(AppSettings::ArgRequiredElseHelp)
            .arg(
                Arg::with_name("key")
                    .help("The key to look for in the config store.")
                    .index(1)
                    .required(true)
            )
            .arg(
                Arg::with_name("forceString")
                    .help("If set forces the variable type to be a string")
                    .short("s")
                    .long("string")
            )
        )
        .subcommand(SubCommand::with_name("set")
            .about("Set a config variable")
            .setting(AppSettings::ArgRequiredElseHelp)
            .arg(
                Arg::with_name("key")
                    .help("The key to used to identified this config variable.")
                    .index(1)
                    .required(true)
                )
            .arg(
                Arg::with_name("value")
                    .help("The value to set for this config variable.")
                    .index(2)
                    .required(true)
                )
        )
}

pub async fn handle(opt: Option<&ArgMatches<'static>>, json_output: bool) -> NirahResult<()> {
    match opt.unwrap().subcommand() {
        ("get", Some(matches)) => {
            let key = value_t_or_exit!(matches, "key", VariableKey);
            let req = RpcRequest::GetConfig { key };
            trace!("Request: {:?}", &req);
            print_response(req, json_output).await
        },
        ("set", Some(matches)) => {
            let key = value_t_or_exit!(matches, "key", VariableKey);
            let value = value_t_or_exit!(matches, "value", VariableValue);
            let req = RpcRequest::SetConfig { key, value };
            trace!("Request: {:?}", req);
            print_response(req, json_output).await
        },
        ("list", _) => {
            let req = RpcRequest::AllVariables;
            trace!("Request: {:?}", req);
            let response = get_response(req).await?;
            if json_output {
                println!("{}", serde_json::to_string(&response)?);
                return Ok(());
            }
            match response {
                RpcResponse::AllConfigVariables { vars } => {
                    let mut table_config = TableConfig::default();
                    let mut display_table = vec![];
                    table_config.columns.insert(0, ColumnConfig {
                        header: "Key".into(),
                        align: Align::Left
                    });
                    table_config.columns.insert(1, ColumnConfig {
                        header: "Default Value".into(),
                        align: Align::Left
                    });
                    table_config.columns.insert(2, ColumnConfig {
                        header: "Value".into(),
                        align: Align::Left
                    });
                    table_config.columns.insert(3, ColumnConfig {
                        header: "Description".into(),
                        align: Align::Left
                    });
                    for var in vars {
                        display_table.push(vec![
                           OptionalDisplay(Some(format!("{}", var.key))),
                           OptionalDisplay(var.default.map(|item|format!("{}", item))),
                           OptionalDisplay(var.value.map(|item|format!("{}", item))),
                           OptionalDisplay(var.description.map(|item|format!("{}", item)))
                        ]);
                    }
                    if display_table.len() == 0 {
                        display_table.push(vec![
                            OptionalDisplay(None),
                            OptionalDisplay(None),
                            OptionalDisplay(None)

                        ])
                    }
                    print_table(&display_table, &table_config);
                    Ok(())
                },
                _ => Ok(())
            }
        },
        _ => unreachable!()
    }
}
