import {doc, getFirestore, serverTimestamp, onSnapshot, getDoc, setDoc, collection, updateDoc} from "firebase/firestore";
import {app, auth} from "auth";
import $ from "jquery";
const db = getFirestore(app);

// Determine if user has admin ability and if so, perform the callback
// Try to write to a document that is protected by rules in firestore rules.
// /authcheck/{userid}
export async function ifAdmin(callback) {
    const currentUser = auth.currentUser;
    // If not logged in, then for sure not an admin.
    if (!currentUser) {
        return;
    }
    try {
        await setDoc(doc(db, "authcheck", currentUser.uid), {
            name: currentUser.displayName,
            email: currentUser.email,
            timestamp: serverTimestamp()
        });
        // Success so perform the callback
        callback();
    } catch(error) {
        // Really not an error, just that the user is not an admin
        console.error("autheck failed:", error.message);
    }
}
