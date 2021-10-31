/**
 * Implements an assembly REPL.
 *
 * Based on https://github.com/kuroko-lang/kuroko-lang.github.io/blob/master/base.js.
 */

var {REPLBackend} = wasm_bindgen;
/** @type {REPLBackend | null} */
var backend = null;
var blockCounter = 0;
var codeHistory = [];
var historySpot = 0;
var scrollToBottom;

async function getBackend() {
    if (!backend) {
        await wasm_bindgen('pkg/website_bg.wasm');
        backend = REPLBackend.new();
    }
    return backend;
}


document.getElementById("container").innerText = "";
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

    getBackend().then(function(backend) {
        const newOutput = document.createElement("pre");
        newOutput.className = "repl";
        newOutput.appendChild(document.createTextNode('=> ' + backend.interpret_assembly(value)));
        document.getElementById("container").appendChild(newOutput);

        currentEditor = createEditor();
    });
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
    editor.commands.bindKey("Return", runCode);
    editor.commands.bindKey("Up", historyBackIfOneLine);
    editor.commands.bindKey("Down", historyForwardIfOneLine);
    editor.focus();
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

currentEditor = createEditor();