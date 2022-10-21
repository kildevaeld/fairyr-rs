// import { TEST } from "./other";

import React, { useState, useEffect } from "react";
import { createRoot } from "react-dom/client";
import image from "./assets/img.png";
import { createStitches } from "@stitches/react";
import { TEST } from "./other.js";
import Ky from "ky";
import { animated } from "react-spring";

const { styled } = createStitches({});

const Test = styled("div", {
  backgroundColor: "gray",
});

function App() {
  const [count, setCount] = useState(0);

  useEffect(() => {
    Ky.get("/src/main.tsx")
      .text()
      .then((resp) => console.log(resp));
  }, []);

  return (
    <animated.div>
      <Test>
        <img src={image} width="200px" />
        <h1>
          Hello: {count} - {process.env.NODE_ENV} {TEST}
        </h1>
        <button onClick={() => setCount(count + 1)}>Click</button>
      </Test>
    </animated.div>
  );
}

let root = createRoot(document.body.querySelector("#root"));

root.render(<App />);
