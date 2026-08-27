import { initializeApp, getApps, getApp } from "firebase/app";
import {
    getAuth,
    signInWithPopup,
    GoogleAuthProvider,
    signOut,
    onAuthStateChanged
} from "firebase/auth";
// Set up Google Auth Provider
const provider = new GoogleAuthProvider();
// Firebase configuration: This key is specific but NOT secret
const not_secret = "AIzaSyAPG10eyt_Wx8qZGEmXP7-q0j0W73W7EWQ";
const firebaseConfig = {
    apiKey: not_secret,
    authDomain: "holyballs.games",
    projectId: "holyballs-2beff",
    storageBucket: "holyballs-2beff.firebasestorage.app",
    messagingSenderId: "826762095856",
    appId: "1:826762095856:web:ba1a7faf95b8124ab36635"
};

let loggedInCallback;
let loggedOutCallback;
export const app = getApps().length === 0 ? initializeApp(firebaseConfig) : getApp();
export const auth = getAuth(app);

export function setCallbacks(callbackLoggedIn, callbackLoggedOut) {
    loggedInCallback = callbackLoggedIn;
    loggedOutCallback = callbackLoggedOut;
}

// Initialize Firebase Core and Auth

export const login = async () => {
    try {
        const result = await signInWithPopup(auth, provider);
        // This gives you a Google Access Token to access Google APIs if needed
        const credential = GoogleAuthProvider.credentialFromResult(result);
        const token = credential.accessToken;
        // The signed-in user info
        const user = result.user;
        console.log("Logged in user:", user.displayName);
        return user;
    } catch (error) {
        console.error("Authentication Error:", error.message);
    }
};
export const logout = async () => {
    try {
        await signOut(auth);
        console.log("Log In");
    } catch (error) {
        console.error("Logout Error:", error.message);
    }
};
onAuthStateChanged(auth, (user) => {
    if (user) {
        console.log(`User ${user.displayName} is active.`);
        if (loggedInCallback) {
            loggedInCallback(user.displayName);
        }
        // setupMenu().then(() => {
        //     getUserScores();
        // });
    } else {
//        console.log("No user signed in.");
        // setupMenu().then();
        if (loggedOutCallback) {
            loggedOutCallback();
        }
    }
});
