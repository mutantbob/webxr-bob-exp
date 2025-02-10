Bob's experiments with WebXR.

[Three.js VR](https://threejs.org/docs/#manual/en/introduction/How-to-create-VR-content)

To run this.

```
# build the first Rust WASM library (sierpinski geometry)
(cd rust/sierpinski && wasm-pack build --target web)

# build the second Rust WASM library (all-Rust WebXR example application)
(cd rust/triangle && RUSTFLAGS="--cfg web_sys_unstable_apis " wasm-pack build --target web --dev )

# tiny web server
(cd webroot && python3 -m http.server)& 

#use firefox to make sure that works
firefox http://127.0.0.1:8000/ & 

# after headset reboot or USB connection
adb reverse tcp:8000 tcp:8000 

# easier than typing it into the browser using the VR controllers
adb shell am start -a android.intent.action.VIEW -d http://127.0.0.1:8000 
```
