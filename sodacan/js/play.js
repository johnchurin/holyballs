import {doc, getFirestore, getDoc, setDoc} from "firebase/firestore";
import {app, auth} from "auth";
import init, { sound, load, play, end_play } from "../generated/holyballs_wasm.js";
import $ from "jquery";
const db = getFirestore(app);
let init_done = false;
$( document ).ready(function() {
    init()
        .then(() => {
            init_done = true;
            $(".play").removeClass("disabled-link");
        })
        .catch(error => {
            console.error("Failed to initialize WASM module:", error);
        })
    ;
    $("#closeBtn").on("click", function() {
        end_play();
        cleanup_after_play();
        if (document.exitFullscreen) {
            document.exitFullscreen().then(); // Modern standard
        }
        console.log("Exiting Game");
    });
    $("#fullscreenContainer").on("fullscreenchange", fullscreenchangeHandler);
});
export async function setupMenu() {
    let tbody = $("#scores tbody");
    // Clear previous entries
    tbody.empty();
    const gamesRef = doc(db, "menus", navigator.language);
    const gameSnap = await getDoc(gamesRef);
    if (gameSnap.exists()) {
        $.each(gameSnap.data().games, function(index, game) {
            tbody.append(
                '<tr>' +
                '<td>' +
                '<a href="#" title="Play this game" class="text-decoration-none play" data-game="' +
                game +
                '">' +
                '<img src="images/play.png" alt="Play">&nbsp' +
                game +
                '</a>' +
                '</td>' +
                '</tr>');
        });
        if (!init_done) {
            $(".play").addClass("disabled-link");
        }
        $(".play").on("click", function() {
            $(".play").addClass("disabled-link");
            // const spinner = document.getElementById("spinner");
            // spinner.style.display = "inline";
            let gameName = $(this).attr('data-game');
            startGame(gameName);
        });
    }
}

// Table is already populated with the menu, we just add score data to the corresponding rows.
export function getUserScores() {
    console.log( "in getUserScores" );
    const currentUser = auth.currentUser;

    if (!currentUser) {
        console.error("You must be logged in to fetch data!");
        return;
    }
    // Iterate through the tbody rows and find the scores, if any, for the game
    let rows = $("#scores tbody tr");
    rows.each(function( index, row) {
        let key = $(this).text().trim();
        console.log("Field: ", "." + key + ".");
        const scoresRef = doc(db, "users", currentUser.uid, "games", key);
        getDoc(scoresRef).then((snap)=> {
            if (snap.exists()) {
                console.log("Document data:", snap.data());
                $(this).append("<td class='text-end'>" + snap.data().lastScore + "</td>");
                $(this).append("<td class='text-end'>" + snap.data().highestScore + "</td>");
            } else {
                console.log("No such document!");
            }
        });
    });
}

export function startGame(gameName) {
    $(".play").addClass("disabled-link");
    console.log("In startGame: ", gameName);
    const container = document.getElementById("fullscreenContainer");
    container.requestFullscreen().catch(err => {
        console.error("Error attempting to enable fullscreen:", err);
    });
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
    canvas.addEventListener('contextmenu', (event) => {
        event.preventDefault();
    });
    canvas.focus();
    console.log("Focus set");
}
function cleanup_after_play() {
    console.log("cleanup_after_play");
    const container = document.getElementById("fullscreenContainer");
    const spinner = document.getElementById("spinner");
    const playLabel = document.getElementById("playLabel");
//    spinner.style.display = "none";
//    playLabel.style.display = "inline";
    container.style.display = "none";
    $(".play").removeClass("disabled-link");
    // if (container.exitFullscreen) {
    //     container.exitFullscreen().then(r => {}); // Modern standard
    // }

}
export function fullscreenchangeHandler(event) {
    console.log("Closing: ", event.target.id);
    if (document.fullscreenElement) {
        console.log(`entered fullscreen mode.`);
    } else {
        console.log("Leaving fullscreen mode.");
        end_play();
        cleanup_after_play();
    }
}

