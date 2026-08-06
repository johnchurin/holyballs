export function console_message(msg) {
    console.log(msg);
}
    export function game_ended() {
    console.log("js: Game Ended");
    if (document.fullscreenElement) {
        document.exitFullscreen().then();
    }
 }
