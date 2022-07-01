* To use ShareArrayBuffer (needed by usb driver module) CORS needs to be set as described here: https://stackoverflow.com/questions/68592278/sharedarraybuffer-is-not-defined
  * Gulp has been modified via middleware to achieve this
* To use modules with web workers, the web worker needs to created with the option `module`
* The imported `read_samples()` function can't be used from a web worker