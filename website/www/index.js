// implements a REPL based on ace.js
//
// Heavily based on https://github.com/kuroko-lang/kuroko-wasm-repl

import {REPLBackend} from "../pkg/website.js";

document.getElementById("container").innerText = "";
let scrollToBottom;
const codeHistory = [];
let blockCounter = 0;
let historySpot = 0;
let currentEditor = createEditor();
const backend = REPLBackend.new();

/**
 * Runs the code in an Ace editor.
 */
function runCode(editor) {
    const value = editor.getValue();
    if (!codeHistory.length || codeHistory[codeHistory.length - 1] !== value) {
        codeHistory.push(value);
    }
    historySpot = codeHistory.length;
    editor.renderer.off("afterRender", scrollToBottom);
    editor.setReadOnly(true);
    editor.renderer.$cursorLayer.element.style.display = "none";
    const frozenEditor = document.createElement("pre");
    frozenEditor.className = "lines";
    const lines = editor.container.getElementsByClassName("ace_line");
    const len = lines.length;
    for (var i = 0; i < len; i++) {
        /* because detaching this apparently pops it ? */
        const child = lines[0];
        const lineNumber = document.createElement("a");
        lineNumber.href = "#_" + blockCounter + "_" + (i + 1);
        child.id = "_" + blockCounter + "_" + (i + 1);
        child.prepend(lineNumber);
        frozenEditor.appendChild(child);
    }
    blockCounter++;
    editor.container.remove();
    editor.destroy();
    document.getElementById("container").appendChild(frozenEditor);

    const state = backend.interpret_assembly(value);

    const newOutput = document.createElement("pre");
    newOutput.className = "repl";
    newOutput.appendChild(document.createTextNode('=> ' + state));
    document.getElementById("container").appendChild(newOutput);

    currentEditor = createEditor();
}

/**
 * Ace command to perform smart enter-key handling.
 *
 * If the text ends in a colon, or if there are multiple lines and the last line
 * of the editor is not blank or all spaces, a line feed will be inserted at the
 * current cursor position. Otherwise, code will be executed and this editor
 * will be marked readonly and a new one will be created after the interpreter
 * returns.
 */
function enterCallback(editor) {
    const value = editor.getValue();
    if ((value.endsWith(":") || value.endsWith("\\")) || (value.split("\n").length > 1 && value.replace(/.*\n */g,"").length > 0)) {
        editor.insert("\n");
        return;
    }
    runCode(editor);
}

function historyBackIfOneLine(editor) {
    const value = editor.getValue();
    if (value.split("\n").length == 1 && codeHistory.length > 0) {
        editor.setValue(codeHistory[historySpot-1],1);
        historySpot--;
        if (historySpot == 0) historySpot = 1;
    } else {
        const current = editor.getCursorPosition();
        editor.moveCursorTo(current.row - 1, current.column, true);
    }
}

function historyForwardIfOneLine(editor) {
    const value = editor.getValue();
    if (value.split("\n").length == 1 && codeHistory.length > 0) {
        if (historySpot == codeHistory.length) {
            editor.setValue('',1);
        } else {
            editor.setValue(codeHistory[historySpot],1);
            historySpot++;
        }
    } else {
        const current = editor.getCursorPosition();
        editor.moveCursorTo(current.row + 1, current.column, true);
    }
}

/**
 * Builds an Ace editor.
 */
function createEditor() {
    const newDiv = document.createElement("div");
    newDiv.className = "editor";
    document.getElementById("container").appendChild(newDiv);
    const editor = ace.edit(newDiv, {
        minLines: 1,
        maxLines: 1000,
        highlightActiveLine: false,
        showPrintMargin: false,
        useSoftTabs: true,
        indentedSoftWrap: false,
        wrap: true
    });
    //   editor.setTheme("ace/theme/sunsmoke");
    //   editor.setBehavioursEnabled(false);
    //   editor.session.setMode("ace/mode/kuroko");
    editor.commands.bindKey("Return", enterCallback);
    editor.commands.bindKey("Up", historyBackIfOneLine);
    editor.commands.bindKey("Down", historyForwardIfOneLine);
    editor.focus();
    scrollToBottom = editor.renderer.on('afterRender', function() {
        newDiv.scrollIntoView();
    });
    return editor;
}

function addText(mode, text) {
    const newOutput = document.createElement("pre");
    newOutput.className = mode;
    newOutput.appendChild(document.createTextNode(text));
    if (!text.length) newOutput.appendChild(document.createElement("wbr"));
    document.getElementById("container").appendChild(newOutput);
}

function insertCode(code) {
    currentEditor.setValue(code,1);
    window.setTimeout(() => runCode(currentEditor), 100);
    return false;
}
