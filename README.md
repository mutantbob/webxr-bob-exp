Bob's experiments with WebXR.

[Three.js VR](https://threejs.org/docs/#manual/en/introduction/How-to-create-VR-content)

To run this.

```
# tiny web server
(cd webroot && python3 -m http.server)& 

#use firefox to make sure that works
firefox http://127.0.0.1:8000/ & 

# after headset reboot or USB connection
adb reverse tcp:8000 tcp:8000 

# easier than typing it into the browser using the VR controllers
adb shell am start -a android.intent.action.VIEW -d http://127.0.0.1:8000 
```
