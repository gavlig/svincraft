# SvinCraft

![svincraft_header](https://github.com/gavlig/svincraft/blob/master/assets/readme/header.gif)

### A first person game prototype with mechanics inspired by StarCraft
Player can mine resources with hand-drill, spawn new npcs and give them move and resource mining commands.
There is also reaction to collision with environment for drill, visual indication of the amount of mined resources,
a little touch on drill animation when left mouse button is released and indication that mining capacity is full.


The purpose of the project was to see how fast a prototype can be made using only freely available assets,  
unmodified [Bevy Engine](https://bevyengine.org/) and a number of plugins for it to achieve some sort of gameplay  
as fast as possible. (Before any polishing [this version](https://www.youtube.com/watch?v=DIbebxN5p7U) was made in about 3 weeks of working in the evenings)


[![video demo](https://img.youtube.com/vi/mPkblvfRDiU/0.jpg)](https://www.youtube.com/watch?v=mPkblvfRDiU)  
Svincraft running on Steam Deck(see branch `steamdeck` for more info):  
[![steamdeck](https://img.youtube.com/vi/rCx1QrgtoWo/0.jpg)](https://www.youtube.com/watch?v=rCx1QrgtoWo) 

### How to build
0. Make sure you have Rust installed. See https://www.rust-lang.org/
1. `cargo build -r`

### How to run
`cargo run -r`

## Controls
- `WASD` - player movement
- `Shift` - sprint
- `Ctrl` - duck
- `Space` - jump
- `Mouse Left Click (aiming at resource or non selectable entity)` - initiate drilling
- `Mouse Left Click (aiming at selectable entity)` - select entity (currently only npc or base building)
- `Mouse Right Click` - give command to selected entity. (currentl onlyy move/mine resources for npc)
- `Shift` + `o` - switch to pan-orbit camera for more rts-like overview
- `Numpad +` - spawn npc at base building if player is looking at one
- `Numpad *` - spawn stresstest batch with 20 bases and 8 npcs at each base that will instantly start mining resources

## Bevy plugins used
- bevy_rapier3d for all in-game physics (https://github.com/dimforge/bevy_rapier)
- bevy_panorbit_camera for flyover camera (https://github.com/Plonq/bevy_panorbit_camera)
- bevy_fps_controller for first person movement (https://github.com/qhdwight/bevy_fps_controller)
- polyanya for npc pathfinding (https://github.com/vleue/polyanya)
- bevy_hanabi for drilling particles (https://github.com/djeedai/bevy_hanabi)
- bevy_vector_shapes for selection indication (https://github.com/james-j-obrien/bevy_vector_shapes)
- bevy-scene-hook for qol features when working with gltf scenes (https://github.com/nicopap/bevy-scene-hook)
- bevy-inspector-egui for in-game editor and entity list (https://github.com/jakobhellermann/bevy-inspector-egui)
- iyes_perf_ui for fps and other diagnostics on screen (https://github.com/IyesGames/iyes_perf_ui)

## Assets used
- base building: "RTS Military Building 1" (https://skfb.ly/66RNn) by Sabri Ayeş is licensed under Creative Commons Attribution (http://creativecommons.org/licenses/by/4.0/).
- teal mineral resource: "Fantasy Crystal Stone" (https://skfb.ly/oKEyo) by Rzyas is licensed under Creative Commons Attribution (http://creativecommons.org/licenses/by/4.0/).
- purple mineral resource: "Crystal stone (rock)" (https://skfb.ly/6WMAx) by GenEugene is licensed under Creative Commons Attribution-NonCommercial (http://creativecommons.org/licenses/by-nc/4.0/).
- svin npc: "Homework 5.1 - pig bot" (https://skfb.ly/6RZPE) by lucidvoo is licensed under Creative Commons Attribution (http://creativecommons.org/licenses/by/4.0/).
- hand drill: "Millers Falls No. 980 Drill | Game Ready Model" (https://skfb.ly/oyWBn) by kennethtzh is licensed under Creative Commons Attribution-NonCommercial (http://creativecommons.org/licenses/by-nc/4.0/).
(animated by Vladyslav Gapchych)
