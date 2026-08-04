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

async function startGame() {
    const container = document.getElementById("fullscreenContainer");
    const closeBtn = document.getElementById("closeBtn");
    const button1 = document.getElementById("play");
    const spinner = document.getElementById("spinner");
    button1.disabled = true;
    container.style.display = "block";
    spinner.style.display = "block";
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
}
