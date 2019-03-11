/// <reference path="../lucis/pkg/lucis_bg.d.ts" />

import { Scene, Raytracer } from '../lucis/pkg'

// Initial warm_up run, makes the next run go much faster
let iScene = new Scene()
let ia = iScene.create_node('a')
let ir = new Raytracer(iScene)
ir.render(1, 1)

let test_scene = new Scene()

let a = test_scene.create_node('a')
// let b = test_scene.create_node('b')
// let c = test_scene.create_node('c')

a.translate(0, 0, -10)
// b.translate(0.2, 0.2, -5)
// c.translate(-0.2, -0.2, -10)

// a.add_child(b)
// b.add_child(c)

let raytracer = new Raytracer(test_scene)
raytracer.render(800, 800)
