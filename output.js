import $importCJS_$nodeModulesFairyReact from "/node_modules/.fairy/react";
const React = $importCJS_$nodeModulesFairyReact;
const useState = $importCJS_$nodeModulesFairyReact.useState;
const useEffect = $importCJS_$nodeModulesFairyReact.useEffect;
import $importCJS_$nodeModulesFairyReactDomClient from "/node_modules/.fairy/react-dom/client";
const createRoot = $importCJS_$nodeModulesFairyReactDomClient.createRoot;
const image = "/src/assets/img.png";
import { createStitches } from "/node_modules/.fairy/@stitches/react";
import { TEST } from "./other.js";
const Ky = "/src/node_modules/.fairy/ky";
console.log(React);
const { styled  } = createStitches({});
const Test = styled("div", {
    backgroundColor: "gray"
});
function App() {
    const [count, setCount] = useState(0);
    useEffect(()=>{
        Ky.get("/src/main.tsx").text((resp)=>console.log(resp));
    }, []);
    return /*#__PURE__*/ React.createElement(Test, null, /*#__PURE__*/ React.createElement("img", {
        src: image,
        width: "200px"
    }), /*#__PURE__*/ React.createElement("h1", null, "Hello: ", count, " - ", "development", " ", TEST), /*#__PURE__*/ React.createElement("button", {
        onClick: ()=>setCount(count + 1)
    }, "Click"));
}
let root = createRoot(document.body.querySelector("#root"));
root.render(/*#__PURE__*/ React.createElement(App, null));
console.log("Hello, World!", React);

