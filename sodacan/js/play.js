import {doc, getFirestore, onSnapshot, getDoc, setDoc, collection, updateDoc} from "firebase/firestore";
import {app, auth} from "auth";
import init, { sound, play, end_play } from "../generated/holyballs_wasm.js";
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
        if (document.exitFullscreen) {
            document.exitFullscreen().then(); // Modern standard
        }
        console.log("Exiting Game");
    });
    $(document).on("fullscreenchange", fullscreenchangeHandler);
});
let unsubscribe;
function gameRow(game) {
    return '<td><a href="#" title="Play this game" class="text-decoration-none play"' +
    '">' +
    '<img src="images/play.png" alt="Play">&nbsp' +
    game +
    '</a>' +
    '</td>';

}
export async function setupMenu() {
    if (unsubscribe) {
        unsubscribe();
        unsubscribe = undefined;
    }

    let tbody = $("#scores tbody");
    // Clear previous entries
    tbody.empty();
    const gamesRef = doc(db, "menus", navigator.language);
    const gameSnap = await getDoc(gamesRef);
    if (gameSnap.exists()) {
        $.each(gameSnap.data().games, function(index, game) {
            tbody.append(
                '<tr data-game="' +
                game +
                '">' +
                gameRow(game) +
                '</tr>');
        });
        if (!init_done) {
            $(".play").addClass("disabled-link");
        }
        $(".play").on("click", function() {
            $(".play").addClass("disabled-link");
            // const spinner = document.getElementById("spinner");
            // spinner.style.display = "inline";
            let gameName = $(this).parent().parent().attr('data-game');
            startGame(gameName);
        });
    }
}
export function updateScore(score) {
    let parts = score.split(",")
    console.log("Score for game:", parts[0], "is", parts[1]);

    const currentUser = auth.currentUser;
    if (!currentUser) {
        displayScores(parts[0], parts[1]);
        return;
    }
    // User is logged in so store the scores
    const docRef = doc(db, "users", currentUser.uid, "games", parts[0]);
    let highest;
    // Get current scores and update as appropriate
    getDoc(docRef).then((snap)=> {
        if (snap.exists()) {
            highest = snap.data().highestScore;
            if (Number(parts[1]) > Number(highest)) {
                highest = parts[1];
            }
            updateDoc(docRef, {
                lastScore: parts[1],
                highestScore: highest
            }).then();
        } else {
            console.log("No such document!");
        }
    });
}
export function subscribeToScores() {
    console.log( "in getUserScores" );
    const currentUser = auth.currentUser;

    if (!currentUser) {
        console.error("You must be logged in to fetch data!");
        return;
    }
    const scoresRef = collection(db, "users/" + currentUser.uid + "/games");
    unsubscribe = onSnapshot(scoresRef, (snap) => {
        snap.docs.forEach((doc) => {
            displayScores(doc.id, doc.data().lastScore, doc.data().highestScore);
//            console.log("Document ID:", doc.id, "Data:", doc.data());
        });
    }, (error) => {
        console.error("Error listening to document: ", error);
    });

}
// Table is already populated with the menu, we just add score data to the corresponding rows.
export function displayScores(game, lastScore, highestScore) {
    // Iterate through the tbody rows and find the scores, if any, for the game
    let rows = $("#scores tbody tr");
    rows.each(function( index, row) {
        let key = $(this).attr('data-game')
        if (key===game) {
            $(this).empty();
            $(this).append(gameRow(game));
            $(this).append("<td class='text-end'>" + lastScore + "</td>");
            if (highestScore) {
                $(this).append("<td class='text-end'>" + highestScore + "</td>");
            }
            $(this).on("click", function() {
                $(".play").addClass("disabled-link");
                startGame(key);
            });
            return;
        }
    });
}

export function startGame(gameName) {
    $(".play").addClass("disabled-link");
    console.log("In startGame: ", gameName);
    const container = document.getElementById("fullscreenContainer");
    container.requestFullscreen().catch(err => {
        console.error("Error attempting to enable fullscreen:", err);
    });
    // const soundElement = document.getElementById("sound");
    // let soundParam;
    // if (soundElement.checked) {
    //     soundParam = "on";
    // } else {
    //     soundParam = "off";
    // }
    // sound(soundParam);
//    gamename(gameName);
    container.style.display = "block";
    const canvas = document.getElementById("game-canvas");
    canvas.addEventListener('contextmenu', (event) => {
        event.preventDefault();
    });
    canvas.focus();
    console.log("Focus set");
    fetchConfigAndPlay(gameName + ".hb.json", gameName);
}
function fetchConfigAndPlay(filename, gameName)  {
    const url = "config/" + filename;
    console.log("load config file: " + url);
    fetch(url)
        .then(function(response) {
            return response.text();
        })
        .then(function(json) {
            return json;
        })
        .then(function(json) {
            play(json, gameName);
        });
}
function cleanup_after_play() {
    console.log("cleanup_after_play");
    const container = document.getElementById("fullscreenContainer");
    // const spinner = document.getElementById("spinner");
    // const playLabel = document.getElementById("playLabel");
//    spinner.style.display = "none";
//    playLabel.style.display = "inline";
    container.style.display = "none";
    $(".play").removeClass("disabled-link");
}
export function fullscreenchangeHandler(event) {
    if (document.fullscreenElement) {
        console.log(`entered fullscreen mode.`);
    } else {
        console.log("Leaving fullscreen mode. Send end_play to engine");
        event.stopPropagation();
        end_play();
        cleanup_after_play();
    }
}

