# Spotify Egui Web App

A web application built with Rust and `egui` that interfaces with the Spotify API. You can try a live demo at [liamchristian.com/spotify](https://liamchristian.com/spotify).

This application allows users to browse their music library, manage playlists, play tracks, and create image collages from album artwork, all running in the browser through WebAssembly.

## Motivation

This project was born out of a desire for more control over the Spotify user interface. The official Spotify client doesn't allow for viewing multiple panels (like playlists, liked songs, and the music player) simultaneously. This application uses a tileable, windowed approach, making it easier to manage and view different parts of your music library at the same time.

## Features

- **Browse Spotify Library**: View and search your liked songs and playlists.
- **Tileable Window UI**: Open, move, and resize windows for liked songs, playlists, and the music player to customize your layout.
- **Dual View Modes**: Switch between a compact list view and a visual grid view for your tracks and playlists.
- **Integrated Music Player**: Control music playback using an embedded player, powered by the Spotify Web Playback SDK.
- **Album Art Collage Generator**: Create and customize beautiful collages from the album art of your liked songs.
  - Customizable dimensions, gradient direction, and starting corner.
  - Download the generated collage as a PNG image.
- **Authentication**: Secure OAuth 2.0 authentication with the Spotify API.
- **UI**: Built with `egui` and `eframe`, providing a responsive and efficient user interface.

## Tech Stack

- **Core Logic**: Rust
- **UI Framework**: `egui` and `eframe`
- **WebAssembly**: Compiled to WASM using `wasm-pack` for in-browser execution.
- **API Interaction**: Direct calls to the Spotify Web API using `reqwest`.
- **Playback**: Integration with the Spotify Web Playback SDK.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- A local web server (e.g., `python -m http.server`, `npx serve`)

### Spotify API Setup

1.  Go to the [Spotify Developer Dashboard](https://developer.spotify.com/dashboard/) and create a new application.
2.  Note your **Client ID**.
3.  Go to "Edit Settings" for your app and add a **Redirect URI**. For local development, you can use `http://localhost:8080/` (or whichever port you will use).
4.  You will need to configure the application with your Client ID. This is likely handled in one of the JavaScript files in the `pkg` directory, such as `auth.js` or `intializer.js`.

### Running the Application

1.  **Build the WebAssembly Package**:
    Compile the Rust code into a WebAssembly package.

    ```sh
    wasm-pack build --target web
    ```

    This command compiles the `src` directory and places the output in the `pkg` directory.

2.  **Serve the Application**:
    The `pkg` directory is the web root. Serve it using a simple local web server.

    ```sh
    cd pkg
    python -m http.server 8080
    ```

3.  **Open in Browser**:
    Navigate to `http://localhost:8080` in your web browser. You will be prompted to log in with your Spotify account to grant the necessary permissions.
