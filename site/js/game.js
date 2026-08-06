import init, { execute } from "../generated/holyballs.js";
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
var initDone = false;
async function startGame() {
    const container = document.getElementById("fullscreenContainer");
    const closeBtn = document.getElementById("closeBtn");
    const button1 = document.getElementById("play");
//    const playLabel = document.getElementById("playLabel");
    const spinner = document.getElementById("spinner");
    button1.disabled = true;
    spinner.style.display = "inline";
//    playLabel.style.display = "inline";
    // We only need to init once, but it must be after some user input so now is a good time.
    if (!initDone) {
        await init();
        initDone = true;
    }
    closeBtn.onclick = () => {
        const args = ["js", "exit"];
        execute(args);
        console.log("Exiting Game");
    };
    container.requestFullscreen().catch(err => {
        console.error("Error attempting to enable fullscreen:", err);
    });

    container.addEventListener("fullscreenchange", fullscreenchangeHandler);

    // Resume audio conext after user input
    // if (window.AudioContext || window.webkitAudioContext) {
    //     const ctx = new (window.AudioContext || window.webkitAudioContext)();
    //     console.log(ctx.state);
    //     if (ctx.state === 'suspended') {
    //         await ctx.resume();
    //     }
    // }
    const sound = document.getElementById("sound");
    const level = document.getElementById("level").value;
    const args = ["js", "start", "-l", String(level)];
    if (sound.checked) {
        args.push("-s");
    }
    execute(args);
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