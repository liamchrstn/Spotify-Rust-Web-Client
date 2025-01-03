# Project Overview

This project interacts with the Spotify API to provide a user interface for managing playlists and music playback.  It leverages Rust/Wasm for performance and a Python server for handling API requests.

## Detailed File Descriptions


**Rust/Wasm:**

- **src/lib.rs:** This file contains the core Rust logic for interacting with the Spotify API. It defines data structures for representing playlists, tracks, and user information. It also implements functions for fetching data from the Spotify API, managing user authentication, and handling playback controls. This library is compiled to WebAssembly (Wasm) for use in the web browser.

- **src/main.rs:** This file serves as the entry point for the Wasm module. It initializes the EGUI-based user interface, handles user interactions, and manages communication with the JavaScript frontend. It likely contains code for rendering playlists, displaying track information, and handling playback events.


**Python Server:**

- **server.py:** This Python script acts as a backend server that handles communication between the Wasm module and the Spotify API. It likely includes functionality for handling OAuth 2.0 authentication with Spotify, proxying API requests, and potentially caching data to improve performance.  This server isolates API keys and secrets from the client-side code.


**Web Frontend:**

- **pkg/index.html:** This HTML file sets up the basic structure of the web page and loads the necessary JavaScript files, including the Wasm module and the EGUI library. It defines the container element where the EGUI-based user interface will be rendered.

- **pkg/spotify_egui.js:** This JavaScript file handles loading and initializing the Wasm module compiled from `src/main.rs`. It provides the glue code for integrating the Wasm module with the web frontend, allowing the Rust/Wasm code to interact with the browser's DOM and handle user events. It also manages communication with the Python server.


## Architecture and Connections

The application follows a client-server architecture with a Rust/Wasm frontend and a Python backend.

1. **User Interaction:** The user interacts with the GUI rendered in the browser by `pkg/spotify_egui.js` which uses the EGUI library.

2. **Frontend Logic:**  `pkg/spotify_egui.js` handles user interactions and communicates with the `server.py` for data fetching and other server-side operations.

3. **Backend Processing:** `server.py` handles authentication with the Spotify API, processes API requests, and returns data to the Wasm frontend.

4. **Rust/Wasm Core:** The core logic for interacting with Spotify's data structures and API resides in `src/lib.rs`, which is compiled to Wasm and used by `src/main.rs` to manage the application's state and GUI.
