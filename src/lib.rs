#![doc = include_str!("../README.md")]

pub mod models;
pub mod services;

mod client;
mod token_manager;
mod error;

pub use self::{ client::SpotifyClient, token_manager::SpotifyTokenManager, error::* };
