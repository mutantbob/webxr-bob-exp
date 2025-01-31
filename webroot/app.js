import { geometry } from "./sierpinski.js";

// Import three
import * as THREE from 'https://unpkg.com/three/build/three.module.js';

//import { ARButton } from 'https://unpkg.com/three/examples/jsm/webxr/ARButton.js';
import { XRButton } from 'https://unpkg.com/three/examples/jsm/webxr/XRButton.js';

// Import the default VRButton
//import { VRButton } from 'https://unpkg.com/three/examples/jsm/webxr/VRButton.js';

// Make a new scene
let scene = new THREE.Scene();
if (false) {
    // Set background color of the scene to gray
    scene.background = new THREE.Color(0x505050);
}

// Make a camera. note that far is set to 100, which is better for realworld sized environments
let camera = new THREE.PerspectiveCamera(50, window.innerWidth / window.innerHeight, 0.1, 100);
camera.position.set(0, 1.6, 0);
scene.add(camera);

// Add some lights
var light = new THREE.DirectionalLight(0xffffff,0.5);
light.position.set(1, 1, 1).normalize();
scene.add(light);
scene.add(new THREE.AmbientLight(0xffffff,0.5))

// Make a red cube
let cube = new THREE.Mesh(
    //new THREE.BoxGeometry(1,1,1),
    geometry(),
/*    new THREE.MeshLambertMaterial(
	//{color:'red'}
	)*/
    new THREE.MeshStandardMaterial( { color: 0xffffff} )

);
cube.position.set(0, 1.5, -0.5);
scene.add(cube);

let dl = new THREE.DirectionalLight(0xff00ff, 0.8)
dl.position.set(0,20,0)
scene.add(dl)

// Make a renderer that fills the screen
let renderer = new THREE.WebGLRenderer({antialias: true, alpha: true});
renderer.setPixelRatio(window.devicePixelRatio);
renderer.setSize(window.innerWidth, window.innerHeight);
// Turn on VR support
renderer.xr.enabled = true;
// Set animation loop
renderer.setAnimationLoop(render);
// Add canvas to the page
document.body.appendChild(renderer.domElement);

// Add a button to enter/exit vr to the page
//document.body.appendChild(VRButton.createButton(renderer));

// For AR instead, import ARButton at the top
//    import { ARButton } from 'https://unpkg.com/three/examples/jsm/webxr/ARButton.js';
// then create the button
//document.body.appendChild(ARButton.createButton(renderer));
document.body.appendChild(XRButton.createButton(renderer));

// Handle browser resize
window.addEventListener('resize', onWindowResize, false);

function onWindowResize() {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
}

function render(time) {
    // Rotate the cube
//    cube.rotation.y = time / 1000;
    // Draw everything
    renderer.render(scene, camera);
}
