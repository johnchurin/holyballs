export function game_ended() {
    console.log("Game Ended");
    const container = document.getElementById("fullscreenContainer");
    const spinner = document.getElementById("spinner");
    const playLabel = document.getElementById("playLabel");
    if (document.fullscreenElement) {
        document.exitFullscreen().then();
    }
    const button = document.getElementById('play');
    // button.innerText = "Play";
    spinner.style.display = "none";
    playLabel.style.display = "block";
    container.style.display = "none";
    button.disabled = false;
}
