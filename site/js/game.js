import init, { exit_game, start } from "../generated/holyballs.js";
const button = document.getElementById("play");
button.addEventListener("click", startGame);

async function startGame() {
    const container = document.getElementById("fullscreenContainer");
    const closeBtn = document.getElementById("closeBtn");
    const button1 = document.getElementById("play");
    button1.disabled = true;
    closeBtn.onclick = () => {
        exit_game();
        console.log("Exiting Game");
    };
    container.requestFullscreen().catch(err => {
        console.error("Error attempting to enable fullscreen:", err);
    });
//            console.log("start game");
    const button = document.getElementById('play');
    button.innerHTML = "<i class='fa fa-spinner fa-spin'></i>Starting Game";
    await init();
    container.style.display = "block";
    const sound = document.getElementById("sound");
    const level = document.getElementById("level").value;
    start(sound.checked, level);
}
