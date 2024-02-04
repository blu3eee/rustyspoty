# rustyspoty

`rustyspoty` is a Rust crate designed to simplify interacting with the Spotify Web API, focusing on server-to-server authentication scenarios using Spotify's Client Credentials Flow. This project aims to make it easier for developers to integrate Spotify API features into their applications without the overhead of handling authentication flows or token expiration management.

## Features

- **Easy Server-to-Server Authentication**: Implements Spotify's Client Credentials Flow, allowing your server to authenticate with Spotify's API for accessing non-user data.
- **Access Token Management**: Automatically handles the retrieval and renewal of access tokens, ensuring that your application can always authenticate its requests to the Spotify API.
- **Safe and Easy Access to Spotify Data**: Provides a straightforward interface for fetching data from Spotify (e.g., artists, albums, tracks) without worrying about the underlying authentication process.
- **Rate Limit Handling** (Upcoming): Aims to support rate limit awareness, intelligently waiting and retrying requests as needed, or advising the user when to try again.

## Disclaimer

This project is a personal initiative and is neither sponsored by nor associated with Spotify. It is not intended for commercial use and is developed with the sole purpose of making the Spotify API more accessible to developers.

## Getting Started

To use `rustyspoty`, you need to have Spotify client credentials (client ID and client secret). If you do not have these, you can obtain them by registering your application in the [Spotify Developer Dashboard](https://developer.spotify.com/dashboard/).

### Installation

Add `rustyspoty` to your `Cargo.toml`:

```toml
[dependencies]
rustyspoty = { git = "https://github.com/bluesimp1102/rustyspoty.git" }
```

## Usage

```rust
use rustyspoty::SpotifyClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_id = "your_spotify_client_id".to_string();
    let client_secret = "your_spotify_client_secret".to_string();

    let mut spotify_client = SpotifyClient::new(client_id, client_secret);

    // Example: Fetch an album
    let album = spotify_client.get_album("album_id_here").await?;
    println!("Album name: {}", album.name);

    Ok(())
}
```

## Contributing

Contributions to rustyspoty are welcome! Whether it's bug reports, feature requests, or code contributions, please feel free to open an issue or submit a pull request on [GitHub](https://github.com/bluesimp1102/rustyspoty).

## License

This project is licensed under the GNU General Public License Version 3 (GPLv3) - see the [LICENSE](LICENSE) file for details.

For more information on GPLv3, please visit [https://www.gnu.org/licenses/gpl-3.0.html](https://www.gnu.org/licenses/gpl-3.0.html).
