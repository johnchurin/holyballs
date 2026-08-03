export function game_ended() {
    console.log("Game Ended");
    const container = document.getElementById("fullscreenContainer");
    if (document.fullscreenElement) {
        document.exitFullscreen().then();
    }
    const button = document.getElementById('play');
    button.innerHTML = "Play Again";
    container.style.display = "none";
    button.disabled = false;
}
