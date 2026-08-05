import init, { exit_game, start_game } from "../generated/holyballs.js";
const button = document.getElementById("play");
button.addEventListener("click", startGame);
const sound = document.getElementById("sound");
sound.addEventListener('change', function() {
    const soundLabel = document.getElementById("soundLabel");
    if (sound.checked) {
        soundLabel.innerHTML = 'Sounds&nbspOn';
    } else {
        soundLabel.innerHTML = 'Sounds&nbspOff';
    }
});

const elements = document.querySelectorAll('p');

// Loop through each element and add the CSS class
elements.forEach(element => {
    element.classList.add('fs-5');
});

async function startGame() {
    const container = document.getElementById("fullscreenContainer");
    const closeBtn = document.getElementById("closeBtn");
    const button1 = document.getElementById("play");
    const playLabel = document.getElementById("playLabel");
    const spinner = document.getElementById("spinner");
    button1.disabled = true;
    spinner.style.display = "block";
    playLabel.style.display = "none";
    closeBtn.onclick = () => {
        exit_game();
        console.log("Exiting Game");
    };
    container.requestFullscreen().catch(err => {
        console.error("Error attempting to enable fullscreen:", err);
    });
    await init();
    const sound = document.getElementById("sound");
    const level = document.getElementById("level").value;
    start_game(sound.checked, level);
    console.log("In start_game");
    container.style.display = "block";
    const canvas = document.getElementById("game-canvas");
    canvas.focus();
    console.log("Focus set");
}
