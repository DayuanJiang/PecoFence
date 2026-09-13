import "./index.css";
import { Composition, Folder } from "remotion";
import { Promo } from "./Promo";
import { Intro } from "./scenes/Intro";
import { Organize } from "./scenes/Organize";
import { Glass } from "./scenes/Glass";
import { Tabs } from "./scenes/Tabs";
import { Focus } from "./scenes/Focus";
import { Outro } from "./scenes/Outro";
import { FeatureFilm } from "./features/FeatureFilm";
import { LaunchFilm } from "./launch/LaunchFilm";
import { ReviewedFilm } from "./reviewed/ReviewedFilm";

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition id="PecoFence-Reviewed" component={ReviewedFilm}
        durationInFrames={900} fps={30} width={1920} height={1080} />
      <Composition id="PecoFence-Launch" component={LaunchFilm}
        durationInFrames={840} fps={30} width={1920} height={1080} />
      <Composition id="PecoFence-Features" component={FeatureFilm}
        durationInFrames={2250} fps={30} width={1920} height={1080} />
      <Composition id="PecoFence-Promo" component={Promo}
        durationInFrames={900} fps={30} width={1920} height={1080} />
      <Folder name="Scenes">
        <Composition id="01-Clarity" component={Intro} durationInFrames={141} fps={30} width={1920} height={1080} />
        <Composition id="02-Organize" component={Organize} durationInFrames={189} fps={30} width={1920} height={1080} />
        <Composition id="03-Native-Glass" component={Glass} durationInFrames={162} fps={30} width={1920} height={1080} />
        <Composition id="04-Tabs" component={Tabs} durationInFrames={177} fps={30} width={1920} height={1080} />
        <Composition id="05-Focus" component={Focus} durationInFrames={147} fps={30} width={1920} height={1080} />
        <Composition id="06-PecoFence" component={Outro} durationInFrames={144} fps={30} width={1920} height={1080} />
      </Folder>
    </>
  );
};
