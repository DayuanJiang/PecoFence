import {FeatureShot,Callout,Keys} from "./common";

export const Peek=()=> <FeatureShot number={5} title="Reach your files from any app"
  description="Peek brings your groups above the current window—without uncovering the desktop."
  file="peek" steps={[
    {at:0,label:"Keep working in your application"},
    {at:77,label:"Ctrl + Alt + Space",note:"Bring your groups to the front"},
    {at:151,label:"Your groups are above the application"},
    {at:249,label:"Esc to return",note:"Continue where you left off"},
  ]}>
    <Callout from={62} to={160} left={1290} top={545}><Keys labels={["Ctrl","Alt","Space"]}/></Callout>
    <Callout from={234} to={323} left={1280} top={545}><Keys labels={["Esc"]}/></Callout>
  </FeatureShot>;
