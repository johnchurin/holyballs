import init, { sound, load, play, end_play } from "../generated/holyballs_wasm.js";
const button = document.getElementById("play");
button.addEventListener("click", () => {
    const spinner = document.getElementById("spinner");
    spinner.style.display = "inline";
    const container = document.getElementById("fullscreenContainer");
    container.requestFullscreen().catch(err => {
        console.error("Error attempting to enable fullscreen:", err);
    });
    initWasm(startGame);
});
const closeBtn = document.getElementById("closeBtn");
closeBtn.onclick = () => {
    end_play();
    cleanup_after_play();
    if (document.exitFullscreen) {
        document.exitFullscreen().then(); // Modern standard
    }
    console.log("Exiting Game");
};
// Load up the menu
fetchMenu();

const container = document.getElementById("fullscreenContainer");
container.addEventListener("fullscreenchange", fullscreenchangeHandler);
const game_level = document.getElementById("game-level");
let jsonConfig = "none";
game_level.addEventListener("change", (event) => {
    fetchConfig(event.target.value);
});
let initDone = false;
// We only need to init once, but it must be after some user input so now is a good time.
function initWasm(callback) {
    if (initDone) {
        callback();
    } else {
        initDone = true;
        init()
            .then(() => {
                callback();
            })
            .catch(error => {
                console.error("Failed to initialize WASM module:", error);
            })
        ;
    }
}


function fetchMenu() {
    const url = "config/menu.json";
    fetch(url)
        .then(function(response) {
            console.log(response.statusText);
            return response.json();
        })
        .then(function(json) {
            const game_level = document.getElementById("game-level");
            // Populate the select dropdown
            let selected = true;
            json.entries.forEach(item => {
                const option = document.createElement("option");
                option.text = item.display;
                option.value = item.file;
                option.selected = selected;
                selected = false;
                game_level.add(option);
            });
            game_level.selectedIndex = 0;
            // Get the first item loaded.
            const event = new Event('change', {bubbles: true});
            game_level.dispatchEvent(event);
        });
}

function fetchConfig(filename)  {
    const url = "config/" + filename;
    console.log("load config file: " + url);
    fetch(url)
    .then(function(response) {
        return response.text();
    })
    .then(function(json) {
        return json;
    })
    .then(function(json) {
        console.log("Type: " + typeof json);
        jsonConfig = json;
    });
}

function startGame() {
    console.log("In startGame");
    const container = document.getElementById("fullscreenContainer");
    const button1 = document.getElementById("play");
    button1.disabled = true;
    load(jsonConfig);
    const soundElement = document.getElementById("sound");
    let soundParam;
    if (soundElement.checked) {
        soundParam = "on";
    } else {
        soundParam = "off";
    }
    sound(soundParam);
//    gamename(gameName);
    play();
    container.style.display = "block";
    const canvas = document.getElementById("game-canvas");
    canvas.focus();
    console.log("Focus set");
}
function cleanup_after_play() {
    console.log("cleanup_after_play");
   const container = document.getElementById("fullscreenContainer");
    const spinner = document.getElementById("spinner");
    const playLabel = document.getElementById("playLabel");
    const button = document.getElementById('play');
    spinner.style.display = "none";
    playLabel.style.display = "inline";
    container.style.display = "none";
    button.disabled = false;
    // if (container.exitFullscreen) {
    //     container.exitFullscreen().then(r => {}); // Modern standard
    // }

}
function fullscreenchangeHandler(event) {
    console.log("Closing: ", event.target.id);
    if (document.fullscreenElement) {
        console.log(`entered fullscreen mode.`);
    } else {
        console.log("Leaving fullscreen mode.");
        end_play();
        cleanup_after_play();
    }
}