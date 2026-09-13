import {Video} from "@remotion/media";
import {staticFile} from "remotion";
import {Headline, Night} from "./shared";

export const Workspaces = () => <Night>
  <Video src={staticFile("launch/tabs.mp4")} muted
    style={{position: "absolute", width: 1920, height: 1080}}/>
  <Headline>Work. Play. Switch.</Headline>
</Night>;
