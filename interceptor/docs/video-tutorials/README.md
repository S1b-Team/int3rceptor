# Video Tutorials Plan & Scripts

This document outlines the plan for the Int3rceptor video tutorial series. Each section contains a script and storyboard for a short (2-5 minute) video.

## Video 1: Installation & Setup (3 mins)

**Goal**: Guide the user from zero to running Int3rceptor.

| Time | Visual                         | Audio (Script)                                                                                              |
| ---- | ------------------------------ | ----------------------------------------------------------------------------------------------------------- |
| 0:00 | Int3rceptor Logo Animation     | "Welcome to Int3rceptor. In this video, we'll get you set up and running in under 3 minutes."               |
| 0:15 | Terminal showing `git clone`   | "First, clone the repository or download the latest binary release from GitHub."                            |
| 0:45 | Terminal showing `cargo build` | "If building from source, run `cargo build --release`. This might take a moment."                           |
| 1:30 | Running `./interceptor`        | "Now, start the application. You'll see it listening on port 8080."                                         |
| 2:00 | Browser Settings               | "Configure your browser proxy to point to localhost:8080. We recommend using FoxyProxy for easy switching." |
| 2:30 | Dashboard in Browser           | "Open `http://localhost:3000`. If you see this dashboard, you're ready to go!"                              |

## Video 2: Your First Interception (4 mins)

**Goal**: Teach the user how to capture and inspect traffic.

| Time | Visual                          | Audio (Script)                                                                                     |
| ---- | ------------------------------- | -------------------------------------------------------------------------------------------------- |
| 0:00 | Dashboard Overview              | "Now that we're running, let's capture some traffic."                                              |
| 0:30 | Browser visiting `example.com`  | "Navigate to a website. Notice how requests instantly appear in the Traffic tab."                  |
| 1:00 | Clicking a request row          | "Click on any request to see the details. On the right, you have the Request and Response panels." |
| 1:45 | Inspecting JSON body            | "Int3rceptor automatically formats JSON responses, making them easy to read."                      |
| 2:30 | Right-click -> Send to Repeater | "Found something interesting? Right-click and send it to the Repeater."                            |
| 3:00 | Repeater Tab                    | "Here you can modify the request and resend it. Let's change this parameter and see what happens." |
| 3:45 | Split screen comparison         | "And there's the response. Fast, simple, and effective."                                           |

## Video 3: Automated Attacks with Intruder (5 mins)

**Goal**: Demonstrate the Fuzzing capabilities.

| Time | Visual                            | Audio (Script)                                                                                       |
| ---- | --------------------------------- | ---------------------------------------------------------------------------------------------------- |
| 0:00 | Intro                             | "Let's take it to the next level with the Intruder."                                                 |
| 0:30 | Sending login request to Intruder | "We've captured a login request. Let's send it to the Intruder to test for weak passwords."          |
| 1:00 | Defining positions                | "Highlight the password field and click 'Add'. This tells Int3rceptor where to inject our payloads." |
| 2:00 | Payload configuration             | "Select 'Sniper' mode and load a list of common passwords."                                          |
| 3:00 | Start Attack                      | "Click Start. Watch as Int3rceptor fires off requests in parallel."                                  |
| 4:00 | Analyzing results                 | "Sort by status code or length. Look, this response is different – we found the valid password!"     |
