//! Users domain: profile persistence; legacy placeholders stay isolated in `legacy`.
pub(crate) mod handler;
pub(crate) mod legacy;
pub(crate) mod model;
pub(crate) mod repository;
pub(crate) mod route;
pub(crate) mod service;

use self::{repository::UserRepository, service::UserService};

pub(crate) type Service = UserService<UserRepository>;
