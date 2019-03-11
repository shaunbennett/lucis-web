# API

### Ideal API
```js
let rt = new Raytracer()

let sphere = raytracer.create_node(Sphere)
sphere.rotate('x', 90)
sphere.scale(2, 2, 2)
sphere.translate(1, 3, 5)
```

### Easier to Impl API
```js
let rt = new Raytracer()

let sphere = raytracer.create_node(Sphere)
rt.rotate(sphere, 'x', 90)
rt.scale(sphere, 2, 2, 2)
rt.translate(sphere, 1, 3, 5)
```
