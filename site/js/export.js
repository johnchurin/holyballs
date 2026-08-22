export function console_message(msg) {
    console.log(msg);
}
export function game_ended() {
    console.log("js: Game Ended");
    if (document.fullscreenElement) {
        document.exitFullscreen().then();
    }
}

export function play_sound(file) {
    const sound = new Audio(file);
    sound.play().then(r => {});
}
