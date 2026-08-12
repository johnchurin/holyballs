import init, { sound, load, play, end_play } from "../generated/holyballs_wasm.js";
const button = document.getElementById("play");
button.addEventListener("click", () => {
    const container = document.getElementById("fullscreenContainer");
    container.requestFullscreen().catch(err => {
        console.error("Error attempting to enable fullscreen:", err);
    });
    init_wasm();
    startGame();
});

// Loop through each <p> element and add the CSS class
const elements = document.querySelectorAll('p');
elements.forEach(element => {
    element.classList.add('fs-5');
});
const game_level = document.getElementById("game-level");
game_level.addEventListener("change", function (event) {
    init_wasm();
    fetchConfig(event.target.value);
});

const soundElement = document.getElementById("sound");
soundElement.addEventListener("change", function (event) {
    let soundParam;
    if (event.target.checked) {
        soundParam = "on";
    } else {
        soundParam = "off";
    }
    init_wasm();
    sound(soundParam);
});

const container = document.getElementById("fullscreenContainer");
container.addEventListener("fullscreenchange", fullscreenchangeHandler);

let initDone = false;
// We only need to init once, but it must be after some user input so now is a good time.
function init_wasm() {
    if (!initDone) {
        init().then(() => {
            initDone = true;
        });
    }
}

// Load up the menu
fetchMenu();

function fetchMenu() {
    const url = "config/menu.json";
    let r;
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
        });
}

function fetchConfig(filename) {
    const url = "config/" + filename;
    console.log("load config file: " + url);
    fetch(url).then(function(response) {
        return response.text();
    })
    .then(function(json) {
        return json;
    })
    .then(function(json) {
        console.log("Type: " + typeof json);
        load(json);
    });
}

function startGame() {
    const container = document.getElementById("fullscreenContainer");
    const closeBtn = document.getElementById("closeBtn");
    const button1 = document.getElementById("play");
//    const playLabel = document.getElementById("playLabel");
    const spinner = document.getElementById("spinner");
    button1.disabled = true;
    spinner.style.display = "inline";
    closeBtn.onclick = () => {
        end_play();
        console.log("Exiting Game");
    };
    play();
    console.log("In start_game");
    container.style.display = "block";
    const canvas = document.getElementById("game-canvas");
    canvas.focus();
    console.log("Focus set");
}
function fullscreenchangeHandler(event) {
    // document.fullscreenElement will point to the element that
    // is in fullscreen mode if there is one. If not, the value
    // of the property is null.
    if (document.fullscreenElement) {
        console.log(`entered fullscreen mode.`);
    } else {
        console.log("Leaving fullscreen mode.");
        console.log("js: Game Ended");
        const container = document.getElementById("fullscreenContainer");
        const spinner = document.getElementById("spinner");
        const playLabel = document.getElementById("playLabel");
        const button = document.getElementById('play');
        // button.innerText = "Play";
        spinner.style.display = "none";
        playLabel.style.display = "inline";
        container.style.display = "none";
        button.disabled = false;
    }
}