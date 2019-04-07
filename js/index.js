/// <reference path="../lucis/pkg/lucis_bg.d.ts" />

import {
    Scene as IScene,
    Raytracer as IRaytracer,
    Color as IColor,
    Material as IMaterial,
    Primitive as IPrimitive,
} from '../lucis/pkg'
import * as ace from 'brace'
import 'brace/mode/javascript'
import 'brace/theme/tomorrow_night_eighties'
let editor = ace.edit('code-box');
editor.getSession().setMode('ace/mode/javascript');
editor.getSession().$worker.send("changeOptions", [{asi: true}]);
editor.setTheme('ace/theme/tomorrow_night_eighties');
editor.session.addGutterDecoration

// // Initial warm_up run, makes the next run go much faster
let iScene = new IScene()
let ia = iScene.create_node('a')
let ir = new IRaytracer(iScene)
ir.render(1, 1)
ia.free()
ir.free()

function getLineNumber(err) {
    if (err.lineNumber) {
        return err.lineNumber
    } else {
        try {
            var caller_line = err.stack.split("\n")[1]
            var index = caller_line.indexOf("<anonymous>:")
            var nums = caller_line.slice(index+12, caller_line.length).split(/[:\)]/)
            var lineNumber = +nums[0]
            var columnNumber = +nums[1]
            return lineNumber
        } catch(e) {
            return NaN
        }
    }
}

let renderBtn = document.getElementById("btn-render")
renderBtn.addEventListener('click', () => {
    let Scene = IScene;
    let Raytracer = IRaytracer;
    let Color = IColor;
    let Material = IMaterial;
    let Primitive = IPrimitive;
    let errorTextNode = document.getElementById("error-text")
    try {
        errorTextNode.innerText = ""
        editor.getSession().clearAnnotations();
        eval(editor.getSession().getValue())
    } catch(err) {
        let lineNumber = getLineNumber(err)
        if (lineNumber) {
            let annotation = {
                row: lineNumber - 1,
                column: 0, // Seems to not be very useful
                text: err.name + ' - ' + err.message,
                type: 'error'
            }
            errorTextNode.innerText = err
            editor.getSession().setAnnotations([annotation]);
        }
    }
})


// let test_scene = new Scene()

// let a = test_scene.create_node('a')
// // let b = test_scene.create_node('b')
// // let c = test_scene.create_node('c')

// a.translate(0, 0, -10)
// // b.translate(0.2, 0.2, -5)
// // c.translate(-0.2, -0.2, -10)

// // a.add_child(b)
// // b.add_child(c)

// let raytracer = new Raytracer(test_scene)
// raytracer.render(4000, 4000)
