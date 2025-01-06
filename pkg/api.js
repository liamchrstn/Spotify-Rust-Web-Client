import init, { start, set_access_token } from './spotify_egui.js';
import { getAccessToken } from './auth.js';

async function run() {
    await init();
    await start();
    const accessToken = getAccessToken();
    if (accessToken) {
        set_access_token(accessToken);
    }
    document.querySelector('.pl').style.display = 'none';
    document.querySelector('canvas').style.display = 'block';
}

run();
