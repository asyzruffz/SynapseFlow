use std::{
    collections::BTreeSet,
    net::{IpAddr, SocketAddr},
};

use synapseflow_domain::ModelReference;
use synapseflow_node::NodeProfile;

use crate::error::CliError;

pub(super) fn profile(value: &str) -> Result<NodeProfile, CliError> {
    match value {
        "development" => Ok(NodeProfile::Development),
        "operational" => Ok(NodeProfile::Operational),
        _ => Err(CliError::NodeConfigurationInvalid),
    }
}

pub(super) fn socket_address(value: &str) -> Result<SocketAddr, CliError> {
    value
        .parse()
        .map_err(|_| CliError::NodeConfigurationInvalid)
}

pub(super) fn addresses(values: Vec<String>) -> Result<BTreeSet<IpAddr>, CliError> {
    values
        .into_iter()
        .map(|value| {
            value
                .parse()
                .map_err(|_| CliError::NodeConfigurationInvalid)
        })
        .collect()
}

pub(super) fn models(values: Vec<String>) -> Result<BTreeSet<ModelReference>, CliError> {
    values
        .into_iter()
        .map(|value| ModelReference::parse(value).map_err(|_| CliError::NodeConfigurationInvalid))
        .collect()
}

pub(super) fn number<T: std::str::FromStr>(value: &str) -> Result<T, CliError> {
    value
        .parse()
        .map_err(|_| CliError::NodeConfigurationInvalid)
}

pub(super) fn comma_list(value: String) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}
