import {Callout,FeatureShot} from "./common";
import {timings} from "./timings";

export const Portal=()=> <FeatureShot number={3} title="Put a live folder on your desktop"
  description="Browse a real folder in place. Its contents stay in sync as files change."
  file="portal" steps={[
    {at:0,label:"Connected folder: Project assets"},
    {at:timings.portal["enter-brand"],label:"Open Brand",note:"Browse a subfolder inside the portal"},
    {at:timings.portal["return-parent"],label:"Return to the parent folder"},
    {at:timings.portal["live-folder-update"],label:"A file is added on disk",note:"The portal updates automatically"},
  ]}>
    <Callout from={timings.portal["live-folder-update"]+16} to={380}
      left={1040} top={565}>New file: review-notes.txt</Callout>
  </FeatureShot>;
