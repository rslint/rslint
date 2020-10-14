/*
*   PostInstall Script: Zaid "Nico" Arshad (https://github.com/zaida04)
*/

const http = require("http"); // modules for making HTTP Request
const https = require("https");
const fs = require("fs");
const {version} = require("./package.json"); // Get version info from package.json
const os = require("os"); // Get OS Information

// Base of the download link, github releases url
const releaseURLBase = "https://github.com/RDambrosio016/RSLint/releases/download/v"; 
const destinationDir = "./bin";

function main() {
    try {
        // Check if bin dir exists/readable
        fs.accessSync(destinationDir, fs.constants.F_OK);
    } catch(_) {
        // If the bin dir does not exist/is not readable, then create it
        fs.mkdirSync(destinationDir);
    } finally {
        const method = releaseURLBase.startsWith("https") ? https : http;
        const OSType = os.type();
        const osURLExtension = getOSURLType(OSType);
        // Build download link based on OS type and version taken from package.json
        const releaseURL = releaseURLBase + `${version}/rslint_cli-${osURLExtension}`;

        console.log("\x1b[32m" + `Fetching prebuilt binary from ${releaseURL}`, "\x1b[0m");
        // Make request to download link, the response will contain the redirect link in the headers
        method.get(releaseURL, (redirectResponse) => {
            // When request finishes and we have all the data, continue
            // Make request to the actual binary file link
            method.get(redirectResponse.headers.location, binaryResponse => {
                binaryResponse.pipe(fs.createWriteStream(`${destinationDir}/rslint${osURLExtension === "windows" ? ".exe" : "ps1"}`))
                console.log("\x1b[32m" + `Successfully downloaded prebuilt binaries`, "\x1b[0m")
            });

        });
    }
}

function getOSURLType(os) {
    if (os === 'Windows_NT') return 'windows';
    if (os === 'Linux') return 'linux';
    if (os === 'Darwin') return 'macos';
}

main();