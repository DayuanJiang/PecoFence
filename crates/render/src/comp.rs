#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AmbientLight(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    AmbientLight,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(AmbientLight, CompositionLight, CompositionObject);
impl AmbientLight {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for AmbientLight {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IAmbientLight>();
}
unsafe impl windows_core::Interface for AmbientLight {
    type Vtable = <IAmbientLight as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IAmbientLight as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for AmbientLight {
    const NAME: &'static str = "Windows.UI.Composition.AmbientLight";
}
unsafe impl Send for AmbientLight {}
unsafe impl Sync for AmbientLight {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AnimationDelayBehavior(pub i32);
impl AnimationDelayBehavior {
    pub const SetInitialValueAfterDelay: Self = Self(0);
    pub const SetInitialValueBeforeDelay: Self = Self(1);
}
impl windows_core::imp::TypeKind for AnimationDelayBehavior {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for AnimationDelayBehavior {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.UI.Composition.AnimationDelayBehavior;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.AnimationDelayBehavior",
    );
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AnimationDirection(pub i32);
impl AnimationDirection {
    pub const Normal: Self = Self(0);
    pub const Reverse: Self = Self(1);
    pub const Alternate: Self = Self(2);
    pub const AlternateReverse: Self = Self(3);
}
impl windows_core::imp::TypeKind for AnimationDirection {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for AnimationDirection {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.UI.Composition.AnimationDirection;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.AnimationDirection");
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AnimationIterationBehavior(pub i32);
impl AnimationIterationBehavior {
    pub const Count: Self = Self(0);
    pub const Forever: Self = Self(1);
}
impl windows_core::imp::TypeKind for AnimationIterationBehavior {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for AnimationIterationBehavior {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.UI.Composition.AnimationIterationBehavior;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.AnimationIterationBehavior",
    );
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AnimationStopBehavior(pub i32);
impl AnimationStopBehavior {
    pub const LeaveCurrentValue: Self = Self(0);
    pub const SetToInitialValue: Self = Self(1);
    pub const SetToFinalValue: Self = Self(2);
}
impl windows_core::imp::TypeKind for AnimationStopBehavior {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for AnimationStopBehavior {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.UI.Composition.AnimationStopBehavior;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.AnimationStopBehavior");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackEasingFunction(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    BackEasingFunction,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    BackEasingFunction,
    CompositionEasingFunction,
    CompositionObject
);
impl BackEasingFunction {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for BackEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IBackEasingFunction>();
}
unsafe impl windows_core::Interface for BackEasingFunction {
    type Vtable = <IBackEasingFunction as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IBackEasingFunction as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for BackEasingFunction {
    const NAME: &'static str = "Windows.UI.Composition.BackEasingFunction";
}
unsafe impl Send for BackEasingFunction {}
unsafe impl Sync for BackEasingFunction {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BounceEasingFunction(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    BounceEasingFunction,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    BounceEasingFunction,
    CompositionEasingFunction,
    CompositionObject
);
impl BounceEasingFunction {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for BounceEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IBounceEasingFunction>();
}
unsafe impl windows_core::Interface for BounceEasingFunction {
    type Vtable = <IBounceEasingFunction as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IBounceEasingFunction as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for BounceEasingFunction {
    const NAME: &'static str = "Windows.UI.Composition.BounceEasingFunction";
}
unsafe impl Send for BounceEasingFunction {}
unsafe impl Sync for BounceEasingFunction {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CircleEasingFunction(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CircleEasingFunction,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    CircleEasingFunction,
    CompositionEasingFunction,
    CompositionObject
);
impl CircleEasingFunction {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CircleEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICircleEasingFunction>();
}
unsafe impl windows_core::Interface for CircleEasingFunction {
    type Vtable = <ICircleEasingFunction as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICircleEasingFunction as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CircleEasingFunction {
    const NAME: &'static str = "Windows.UI.Composition.CircleEasingFunction";
}
unsafe impl Send for CircleEasingFunction {}
unsafe impl Sync for CircleEasingFunction {}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Color {
    pub A: u8,
    pub R: u8,
    pub G: u8,
    pub B: u8,
}
impl windows_core::imp::TypeKind for Color {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for Color {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"struct(Windows.UI.Color;u1;u1;u1;u1)");
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Color");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColorKeyFrameAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ColorKeyFrameAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    ColorKeyFrameAnimation,
    IAnimationObject,
    ICompositionAnimationBase,
    KeyFrameAnimation,
    CompositionAnimation,
    CompositionObject
);
impl ColorKeyFrameAnimation {
    pub(crate) fn InterpolationColorSpace(&self) -> windows_core::Result<CompositionColorSpace> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).InterpolationColorSpace)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetInterpolationColorSpace(
        &self,
        value: CompositionColorSpace,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetInterpolationColorSpace)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertKeyFrame(
        &self,
        normalizedprogresskey: f32,
        value: Color,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).InsertKeyFrame)(
                windows_core::Interface::as_raw(self),
                normalizedprogresskey,
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertKeyFrameWithEasingFunction<P2>(
        &self,
        normalizedprogresskey: f32,
        value: Color,
        easingfunction: P2,
    ) -> windows_core::Result<()>
    where
        P2: windows_core::Param<CompositionEasingFunction>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).InsertKeyFrameWithEasingFunction)(
                windows_core::Interface::as_raw(self),
                normalizedprogresskey,
                value,
                easingfunction.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn ClearAllParameters(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearAllParameters)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub(crate) fn ClearParameter(&self, key: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
            )
            .ok()
        }
    }
    pub(crate) fn SetColorParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Color,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetColorParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix3x2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix3x2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix4x4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix4x4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetQuaternionParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Quaternion,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetQuaternionParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetReferenceParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScalarParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector3Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector3Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetBooleanParameter(
        &self,
        key: &windows_core::HSTRING,
        value: bool,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBooleanParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Target(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Target)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetTarget(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTarget)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn DelayTime(&self) -> windows_core::Result<windows_time::TimeSpan> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).DelayTime)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDelayTime(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDelayTime)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Duration(&self) -> windows_core::Result<windows_time::TimeSpan> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Duration)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDuration(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDuration)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IterationBehavior(&self) -> windows_core::Result<AnimationIterationBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IterationBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIterationBehavior(
        &self,
        value: AnimationIterationBehavior,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIterationBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IterationCount(&self) -> windows_core::Result<i32> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IterationCount)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIterationCount(&self, value: i32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIterationCount)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn KeyFrameCount(&self) -> windows_core::Result<i32> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).KeyFrameCount)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn StopBehavior(&self) -> windows_core::Result<AnimationStopBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).StopBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetStopBehavior(&self, value: AnimationStopBehavior) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetStopBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertExpressionKeyFrame(
        &self,
        normalizedprogresskey: f32,
        value: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).InsertExpressionKeyFrame)(
                windows_core::Interface::as_raw(this),
                normalizedprogresskey,
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn InsertExpressionKeyFrameWithEasingFunction<P2>(
        &self,
        normalizedprogresskey: f32,
        value: &windows_core::HSTRING,
        easingfunction: P2,
    ) -> windows_core::Result<()>
    where
        P2: windows_core::Param<CompositionEasingFunction>,
    {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).InsertExpressionKeyFrameWithEasingFunction)(
                windows_core::Interface::as_raw(this),
                normalizedprogresskey,
                core::mem::transmute_copy(value),
                easingfunction.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn Direction(&self) -> windows_core::Result<AnimationDirection> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Direction)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDirection(&self, value: AnimationDirection) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDirection)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn DelayBehavior(&self) -> windows_core::Result<AnimationDelayBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation3>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).DelayBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDelayBehavior(
        &self,
        value: AnimationDelayBehavior,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation3>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDelayBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for ColorKeyFrameAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IColorKeyFrameAnimation>();
}
unsafe impl windows_core::Interface for ColorKeyFrameAnimation {
    type Vtable = <IColorKeyFrameAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IColorKeyFrameAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for ColorKeyFrameAnimation {
    const NAME: &'static str = "Windows.UI.Composition.ColorKeyFrameAnimation";
}
unsafe impl Send for ColorKeyFrameAnimation {}
unsafe impl Sync for ColorKeyFrameAnimation {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    CompositionAnimation,
    IAnimationObject,
    ICompositionAnimationBase,
    CompositionObject
);
impl CompositionAnimation {
    pub(crate) fn ClearAllParameters(&self) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).ClearAllParameters)(
                windows_core::Interface::as_raw(self),
            )
            .ok()
        }
    }
    pub(crate) fn ClearParameter(&self, key: &windows_core::HSTRING) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).ClearParameter)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(key),
            )
            .ok()
        }
    }
    pub(crate) fn SetColorParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Color,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetColorParameter)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix3x2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetMatrix3x2Parameter)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix4x4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetMatrix4x4Parameter)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetQuaternionParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Quaternion,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetQuaternionParameter)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetReferenceParameter)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetScalarParameter)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetVector2Parameter)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector3Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetVector3Parameter)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector4,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetVector4Parameter)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetBooleanParameter(
        &self,
        key: &windows_core::HSTRING,
        value: bool,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBooleanParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Target(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Target)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetTarget(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTarget)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<Self>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionAnimation>();
}
unsafe impl windows_core::Interface for CompositionAnimation {
    type Vtable = <ICompositionAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionAnimation {
    const NAME: &'static str = "Windows.UI.Composition.CompositionAnimation";
}
unsafe impl Send for CompositionAnimation {}
unsafe impl Sync for CompositionAnimation {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionAnimationGroup(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionAnimationGroup,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionAnimationGroup, CompositionObject);
impl CompositionAnimationGroup {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn First(
        &self,
    ) -> windows_core::Result<windows_collections::IIterator<CompositionAnimation>> {
        let this = &windows_core::Interface::cast::<IIterable<CompositionAnimation>>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).First)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
impl windows_core::RuntimeType for CompositionAnimationGroup {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionAnimationGroup>();
}
unsafe impl windows_core::Interface for CompositionAnimationGroup {
    type Vtable = <ICompositionAnimationGroup as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionAnimationGroup as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionAnimationGroup {
    const NAME: &'static str = "Windows.UI.Composition.CompositionAnimationGroup";
}
unsafe impl Send for CompositionAnimationGroup {}
unsafe impl Sync for CompositionAnimationGroup {}
impl IntoIterator for CompositionAnimationGroup {
    type Item = CompositionAnimation;
    type IntoIter = windows_collections::BufferedIterator<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(&self)
    }
}
impl IntoIterator for &CompositionAnimationGroup {
    type Item = CompositionAnimation;
    type IntoIter = windows_collections::BufferedIterator<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        windows_collections::BufferedIterator::new(self.First().unwrap())
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionBackdropBrush(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionBackdropBrush,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    CompositionBackdropBrush,
    CompositionBrush,
    CompositionObject
);
impl CompositionBackdropBrush {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionBackdropBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionBackdropBrush>();
}
unsafe impl windows_core::Interface for CompositionBackdropBrush {
    type Vtable = <ICompositionBackdropBrush as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionBackdropBrush as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionBackdropBrush {
    const NAME: &'static str = "Windows.UI.Composition.CompositionBackdropBrush";
}
unsafe impl Send for CompositionBackdropBrush {}
unsafe impl Sync for CompositionBackdropBrush {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CompositionBackfaceVisibility(pub i32);
impl CompositionBackfaceVisibility {
    pub const Inherit: Self = Self(0);
    pub const Visible: Self = Self(1);
    pub const Hidden: Self = Self(2);
}
impl windows_core::imp::TypeKind for CompositionBackfaceVisibility {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for CompositionBackfaceVisibility {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.UI.Composition.CompositionBackfaceVisibility;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.CompositionBackfaceVisibility",
    );
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionBatchCompletedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionBatchCompletedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    CompositionBatchCompletedEventArgs,
    IAnimationObject,
    CompositionObject
);
impl CompositionBatchCompletedEventArgs {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionBatchCompletedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionBatchCompletedEventArgs>();
}
unsafe impl windows_core::Interface for CompositionBatchCompletedEventArgs {
    type Vtable = <ICompositionBatchCompletedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <ICompositionBatchCompletedEventArgs as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionBatchCompletedEventArgs {
    const NAME: &'static str = "Windows.UI.Composition.CompositionBatchCompletedEventArgs";
}
unsafe impl Send for CompositionBatchCompletedEventArgs {}
unsafe impl Sync for CompositionBatchCompletedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CompositionBatchTypes(pub u32);
impl CompositionBatchTypes {
    pub const None: Self = Self(0);
    pub const Animation: Self = Self(1);
    pub const Effect: Self = Self(2);
    pub const InfiniteAnimation: Self = Self(4);
    pub const AllAnimations: Self = Self(5);
}
impl windows_core::imp::TypeKind for CompositionBatchTypes {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for CompositionBatchTypes {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.UI.Composition.CompositionBatchTypes;u4)",
    );
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.CompositionBatchTypes");
}
impl CompositionBatchTypes {
    pub const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}
impl core::ops::BitOr for CompositionBatchTypes {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl core::ops::BitAnd for CompositionBatchTypes {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}
impl core::ops::BitOrAssign for CompositionBatchTypes {
    fn bitor_assign(&mut self, other: Self) {
        self.0.bitor_assign(other.0);
    }
}
impl core::ops::BitAndAssign for CompositionBatchTypes {
    fn bitand_assign(&mut self, other: Self) {
        self.0.bitand_assign(other.0);
    }
}
impl core::ops::Not for CompositionBatchTypes {
    type Output = Self;
    fn not(self) -> Self {
        Self(self.0.not())
    }
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CompositionBorderMode(pub i32);
impl CompositionBorderMode {
    pub const Inherit: Self = Self(0);
    pub const Soft: Self = Self(1);
    pub const Hard: Self = Self(2);
}
impl windows_core::imp::TypeKind for CompositionBorderMode {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for CompositionBorderMode {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.UI.Composition.CompositionBorderMode;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.CompositionBorderMode");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionBrush(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionBrush,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionBrush, IAnimationObject, CompositionObject);
impl CompositionBrush {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionBrush>();
}
unsafe impl windows_core::Interface for CompositionBrush {
    type Vtable = <ICompositionBrush as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionBrush as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionBrush {
    const NAME: &'static str = "Windows.UI.Composition.CompositionBrush";
}
unsafe impl Send for CompositionBrush {}
unsafe impl Sync for CompositionBrush {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionClip(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionClip,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionClip, IAnimationObject, CompositionObject);
impl CompositionClip {
    pub(crate) fn AnchorPoint(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).AnchorPoint)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetAnchorPoint(
        &self,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetAnchorPoint)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn CenterPoint(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CenterPoint)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetCenterPoint(
        &self,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetCenterPoint)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Offset(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Offset)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetOffset(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetOffset)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RotationAngle(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RotationAngle)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAngle(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRotationAngle)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RotationAngleInDegrees(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RotationAngleInDegrees)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAngleInDegrees(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRotationAngleInDegrees)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Scale(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Scale)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetScale(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScale)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn TransformMatrix(&self) -> windows_core::Result<windows_numerics::Matrix3x2> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).TransformMatrix)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetTransformMatrix(
        &self,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTransformMatrix)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionClip {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionClip>();
}
unsafe impl windows_core::Interface for CompositionClip {
    type Vtable = <ICompositionClip as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionClip as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionClip {
    const NAME: &'static str = "Windows.UI.Composition.CompositionClip";
}
unsafe impl Send for CompositionClip {}
unsafe impl Sync for CompositionClip {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionColorBrush(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionColorBrush,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    CompositionColorBrush,
    IAnimationObject,
    CompositionBrush,
    CompositionObject
);
impl CompositionColorBrush {
    pub(crate) fn Color(&self) -> windows_core::Result<Color> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Color)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetColor(&self, value: Color) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetColor)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionColorBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionColorBrush>();
}
unsafe impl windows_core::Interface for CompositionColorBrush {
    type Vtable = <ICompositionColorBrush as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionColorBrush as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionColorBrush {
    const NAME: &'static str = "Windows.UI.Composition.CompositionColorBrush";
}
unsafe impl Send for CompositionColorBrush {}
unsafe impl Sync for CompositionColorBrush {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionColorGradientStop(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionColorGradientStop,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionColorGradientStop, CompositionObject);
impl CompositionColorGradientStop {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionColorGradientStop {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionColorGradientStop>();
}
unsafe impl windows_core::Interface for CompositionColorGradientStop {
    type Vtable = <ICompositionColorGradientStop as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionColorGradientStop as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionColorGradientStop {
    const NAME: &'static str = "Windows.UI.Composition.CompositionColorGradientStop";
}
unsafe impl Send for CompositionColorGradientStop {}
unsafe impl Sync for CompositionColorGradientStop {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CompositionColorSpace(pub i32);
impl CompositionColorSpace {
    pub const Auto: Self = Self(0);
    pub const Hsl: Self = Self(1);
    pub const Rgb: Self = Self(2);
    pub const HslLinear: Self = Self(3);
    pub const RgbLinear: Self = Self(4);
}
impl windows_core::imp::TypeKind for CompositionColorSpace {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for CompositionColorSpace {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.UI.Composition.CompositionColorSpace;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.CompositionColorSpace");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionCommitBatch(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionCommitBatch,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionCommitBatch, CompositionObject);
impl CompositionCommitBatch {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionCommitBatch {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionCommitBatch>();
}
unsafe impl windows_core::Interface for CompositionCommitBatch {
    type Vtable = <ICompositionCommitBatch as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionCommitBatch as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionCommitBatch {
    const NAME: &'static str = "Windows.UI.Composition.CompositionCommitBatch";
}
unsafe impl Send for CompositionCommitBatch {}
unsafe impl Sync for CompositionCommitBatch {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CompositionCompositeMode(pub i32);
impl CompositionCompositeMode {
    pub const Inherit: Self = Self(0);
    pub const SourceOver: Self = Self(1);
    pub const DestinationInvert: Self = Self(2);
    pub const MinBlend: Self = Self(3);
}
impl windows_core::imp::TypeKind for CompositionCompositeMode {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for CompositionCompositeMode {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.UI.Composition.CompositionCompositeMode;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.CompositionCompositeMode",
    );
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionEasingFunction(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionEasingFunction,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    CompositionEasingFunction,
    IAnimationObject,
    CompositionObject
);
impl CompositionEasingFunction {
    pub(crate) fn CreateCubicBezierEasingFunction<P0>(
        owner: P0,
        controlpoint1: windows_numerics::Vector2,
        controlpoint2: windows_numerics::Vector2,
    ) -> windows_core::Result<CubicBezierEasingFunction>
    where
        P0: windows_core::Param<Compositor>,
    {
        Self::ICompositionEasingFunctionStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateCubicBezierEasingFunction)(
                windows_core::Interface::as_raw(this),
                owner.param().abi(),
                controlpoint1,
                controlpoint2,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub(crate) fn CreateLinearEasingFunction<P0>(
        owner: P0,
    ) -> windows_core::Result<LinearEasingFunction>
    where
        P0: windows_core::Param<Compositor>,
    {
        Self::ICompositionEasingFunctionStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateLinearEasingFunction)(
                windows_core::Interface::as_raw(this),
                owner.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub(crate) fn CreateStepEasingFunction<P0>(
        owner: P0,
    ) -> windows_core::Result<StepEasingFunction>
    where
        P0: windows_core::Param<Compositor>,
    {
        Self::ICompositionEasingFunctionStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateStepEasingFunction)(
                windows_core::Interface::as_raw(this),
                owner.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub(crate) fn CreateStepEasingFunctionWithStepCount<P0>(
        owner: P0,
        stepcount: i32,
    ) -> windows_core::Result<StepEasingFunction>
    where
        P0: windows_core::Param<Compositor>,
    {
        Self::ICompositionEasingFunctionStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateStepEasingFunctionWithStepCount)(
                windows_core::Interface::as_raw(this),
                owner.param().abi(),
                stepcount,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub(crate) fn CreateBackEasingFunction<P0>(
        owner: P0,
        mode: CompositionEasingFunctionMode,
        amplitude: f32,
    ) -> windows_core::Result<BackEasingFunction>
    where
        P0: windows_core::Param<Compositor>,
    {
        Self::ICompositionEasingFunctionStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateBackEasingFunction)(
                windows_core::Interface::as_raw(this),
                owner.param().abi(),
                mode,
                amplitude,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub(crate) fn CreateBounceEasingFunction<P0>(
        owner: P0,
        mode: CompositionEasingFunctionMode,
        bounces: i32,
        bounciness: f32,
    ) -> windows_core::Result<BounceEasingFunction>
    where
        P0: windows_core::Param<Compositor>,
    {
        Self::ICompositionEasingFunctionStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateBounceEasingFunction)(
                windows_core::Interface::as_raw(this),
                owner.param().abi(),
                mode,
                bounces,
                bounciness,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub(crate) fn CreateCircleEasingFunction<P0>(
        owner: P0,
        mode: CompositionEasingFunctionMode,
    ) -> windows_core::Result<CircleEasingFunction>
    where
        P0: windows_core::Param<Compositor>,
    {
        Self::ICompositionEasingFunctionStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateCircleEasingFunction)(
                windows_core::Interface::as_raw(this),
                owner.param().abi(),
                mode,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub(crate) fn CreateElasticEasingFunction<P0>(
        owner: P0,
        mode: CompositionEasingFunctionMode,
        oscillations: i32,
        springiness: f32,
    ) -> windows_core::Result<ElasticEasingFunction>
    where
        P0: windows_core::Param<Compositor>,
    {
        Self::ICompositionEasingFunctionStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateElasticEasingFunction)(
                windows_core::Interface::as_raw(this),
                owner.param().abi(),
                mode,
                oscillations,
                springiness,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub(crate) fn CreateExponentialEasingFunction<P0>(
        owner: P0,
        mode: CompositionEasingFunctionMode,
        exponent: f32,
    ) -> windows_core::Result<ExponentialEasingFunction>
    where
        P0: windows_core::Param<Compositor>,
    {
        Self::ICompositionEasingFunctionStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateExponentialEasingFunction)(
                windows_core::Interface::as_raw(this),
                owner.param().abi(),
                mode,
                exponent,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub(crate) fn CreatePowerEasingFunction<P0>(
        owner: P0,
        mode: CompositionEasingFunctionMode,
        power: f32,
    ) -> windows_core::Result<PowerEasingFunction>
    where
        P0: windows_core::Param<Compositor>,
    {
        Self::ICompositionEasingFunctionStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreatePowerEasingFunction)(
                windows_core::Interface::as_raw(this),
                owner.param().abi(),
                mode,
                power,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub(crate) fn CreateSineEasingFunction<P0>(
        owner: P0,
        mode: CompositionEasingFunctionMode,
    ) -> windows_core::Result<SineEasingFunction>
    where
        P0: windows_core::Param<Compositor>,
    {
        Self::ICompositionEasingFunctionStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateSineEasingFunction)(
                windows_core::Interface::as_raw(this),
                owner.param().abi(),
                mode,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    fn ICompositionEasingFunctionStatics<
        R,
        F: FnOnce(&ICompositionEasingFunctionStatics) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            CompositionEasingFunction,
            ICompositionEasingFunctionStatics,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for CompositionEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionEasingFunction>();
}
unsafe impl windows_core::Interface for CompositionEasingFunction {
    type Vtable = <ICompositionEasingFunction as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionEasingFunction as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionEasingFunction {
    const NAME: &'static str = "Windows.UI.Composition.CompositionEasingFunction";
}
unsafe impl Send for CompositionEasingFunction {}
unsafe impl Sync for CompositionEasingFunction {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CompositionEasingFunctionMode(pub i32);
impl CompositionEasingFunctionMode {
    pub const In: Self = Self(0);
    pub const Out: Self = Self(1);
    pub const InOut: Self = Self(2);
}
impl windows_core::imp::TypeKind for CompositionEasingFunctionMode {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for CompositionEasingFunctionMode {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.UI.Composition.CompositionEasingFunctionMode;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.CompositionEasingFunctionMode",
    );
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionEffectFactory(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionEffectFactory,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionEffectFactory, CompositionObject);
impl CompositionEffectFactory {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionEffectFactory {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionEffectFactory>();
}
unsafe impl windows_core::Interface for CompositionEffectFactory {
    type Vtable = <ICompositionEffectFactory as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionEffectFactory as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionEffectFactory {
    const NAME: &'static str = "Windows.UI.Composition.CompositionEffectFactory";
}
unsafe impl Send for CompositionEffectFactory {}
unsafe impl Sync for CompositionEffectFactory {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionGeometricClip(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionGeometricClip,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    CompositionGeometricClip,
    IAnimationObject,
    CompositionClip,
    CompositionObject
);
impl CompositionGeometricClip {
    pub(crate) fn AnchorPoint(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).AnchorPoint)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetAnchorPoint(
        &self,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetAnchorPoint)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn CenterPoint(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CenterPoint)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetCenterPoint(
        &self,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetCenterPoint)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Offset(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Offset)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetOffset(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetOffset)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RotationAngle(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RotationAngle)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAngle(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRotationAngle)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RotationAngleInDegrees(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RotationAngleInDegrees)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAngleInDegrees(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRotationAngleInDegrees)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Scale(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Scale)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetScale(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScale)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn TransformMatrix(&self) -> windows_core::Result<windows_numerics::Matrix3x2> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).TransformMatrix)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetTransformMatrix(
        &self,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTransformMatrix)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Geometry(&self) -> windows_core::Result<CompositionGeometry> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Geometry)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetGeometry<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<CompositionGeometry>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetGeometry)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn ViewBox(&self) -> windows_core::Result<CompositionViewBox> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ViewBox)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetViewBox<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<CompositionViewBox>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetViewBox)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionGeometricClip {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionGeometricClip>();
}
unsafe impl windows_core::Interface for CompositionGeometricClip {
    type Vtable = <ICompositionGeometricClip as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionGeometricClip as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionGeometricClip {
    const NAME: &'static str = "Windows.UI.Composition.CompositionGeometricClip";
}
unsafe impl Send for CompositionGeometricClip {}
unsafe impl Sync for CompositionGeometricClip {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionGeometry(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionGeometry,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionGeometry, IAnimationObject, CompositionObject);
impl CompositionGeometry {
    pub(crate) fn TrimEnd(&self) -> windows_core::Result<f32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TrimEnd)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetTrimEnd(&self, value: f32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetTrimEnd)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn TrimOffset(&self) -> windows_core::Result<f32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TrimOffset)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetTrimOffset(&self, value: f32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetTrimOffset)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn TrimStart(&self) -> windows_core::Result<f32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TrimStart)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetTrimStart(&self, value: f32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetTrimStart)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionGeometry {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionGeometry>();
}
unsafe impl windows_core::Interface for CompositionGeometry {
    type Vtable = <ICompositionGeometry as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionGeometry as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionGeometry {
    const NAME: &'static str = "Windows.UI.Composition.CompositionGeometry";
}
unsafe impl Send for CompositionGeometry {}
unsafe impl Sync for CompositionGeometry {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CompositionGetValueStatus(pub i32);
impl CompositionGetValueStatus {
    pub const Succeeded: Self = Self(0);
    pub const TypeMismatch: Self = Self(1);
    pub const NotFound: Self = Self(2);
}
impl windows_core::imp::TypeKind for CompositionGetValueStatus {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for CompositionGetValueStatus {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.UI.Composition.CompositionGetValueStatus;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.CompositionGetValueStatus",
    );
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionGradientBrush(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionGradientBrush,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    CompositionGradientBrush,
    CompositionBrush,
    CompositionObject
);
impl CompositionGradientBrush {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionGradientBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionGradientBrush>();
}
unsafe impl windows_core::Interface for CompositionGradientBrush {
    type Vtable = <ICompositionGradientBrush as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionGradientBrush as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionGradientBrush {
    const NAME: &'static str = "Windows.UI.Composition.CompositionGradientBrush";
}
unsafe impl Send for CompositionGradientBrush {}
unsafe impl Sync for CompositionGradientBrush {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionLight(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionLight,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionLight, CompositionObject);
impl CompositionLight {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionLight {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionLight>();
}
unsafe impl windows_core::Interface for CompositionLight {
    type Vtable = <ICompositionLight as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionLight as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionLight {
    const NAME: &'static str = "Windows.UI.Composition.CompositionLight";
}
unsafe impl Send for CompositionLight {}
unsafe impl Sync for CompositionLight {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionLinearGradientBrush(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionLinearGradientBrush,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    CompositionLinearGradientBrush,
    CompositionGradientBrush,
    CompositionBrush,
    CompositionObject
);
impl CompositionLinearGradientBrush {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionLinearGradientBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionLinearGradientBrush>();
}
unsafe impl windows_core::Interface for CompositionLinearGradientBrush {
    type Vtable = <ICompositionLinearGradientBrush as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <ICompositionLinearGradientBrush as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionLinearGradientBrush {
    const NAME: &'static str = "Windows.UI.Composition.CompositionLinearGradientBrush";
}
unsafe impl Send for CompositionLinearGradientBrush {}
unsafe impl Sync for CompositionLinearGradientBrush {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionMaskBrush(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionMaskBrush,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionMaskBrush, CompositionBrush, CompositionObject);
impl CompositionMaskBrush {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionMaskBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionMaskBrush>();
}
unsafe impl windows_core::Interface for CompositionMaskBrush {
    type Vtable = <ICompositionMaskBrush as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionMaskBrush as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionMaskBrush {
    const NAME: &'static str = "Windows.UI.Composition.CompositionMaskBrush";
}
unsafe impl Send for CompositionMaskBrush {}
unsafe impl Sync for CompositionMaskBrush {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionNineGridBrush(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionNineGridBrush,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    CompositionNineGridBrush,
    CompositionBrush,
    CompositionObject
);
impl CompositionNineGridBrush {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionNineGridBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionNineGridBrush>();
}
unsafe impl windows_core::Interface for CompositionNineGridBrush {
    type Vtable = <ICompositionNineGridBrush as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionNineGridBrush as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionNineGridBrush {
    const NAME: &'static str = "Windows.UI.Composition.CompositionNineGridBrush";
}
unsafe impl Send for CompositionNineGridBrush {}
unsafe impl Sync for CompositionNineGridBrush {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionObject(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionObject,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionObject, IAnimationObject);
impl CompositionObject {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Compositor)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Dispatcher)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Properties)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).StartAnimation)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).StopAnimation)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationWithIAnimationObject<P0, P2>(
        target: P0,
        propertyname: &windows_core::HSTRING,
        animation: P2,
    ) -> windows_core::Result<()>
    where
        P0: windows_core::Param<IAnimationObject>,
        P2: windows_core::Param<CompositionAnimation>,
    {
        Self::ICompositionObjectStatics(|this| unsafe {
            (windows_core::Interface::vtable(this).StartAnimationWithIAnimationObject)(
                windows_core::Interface::as_raw(this),
                target.param().abi(),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        })
    }
    pub(crate) fn StartAnimationGroupWithIAnimationObject<P0, P1>(
        target: P0,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P0: windows_core::Param<IAnimationObject>,
        P1: windows_core::Param<ICompositionAnimationBase>,
    {
        Self::ICompositionObjectStatics(|this| unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroupWithIAnimationObject)(
                windows_core::Interface::as_raw(this),
                target.param().abi(),
                animation.param().abi(),
            )
            .ok()
        })
    }
    fn ICompositionObjectStatics<
        R,
        F: FnOnce(&ICompositionObjectStatics) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            CompositionObject,
            ICompositionObjectStatics,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for CompositionObject {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionObject>();
}
unsafe impl windows_core::Interface for CompositionObject {
    type Vtable = <ICompositionObject as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionObject as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionObject {
    const NAME: &'static str = "Windows.UI.Composition.CompositionObject";
}
unsafe impl Send for CompositionObject {}
unsafe impl Sync for CompositionObject {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionPropertySet(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionPropertySet,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionPropertySet, IAnimationObject, CompositionObject);
impl CompositionPropertySet {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<Self> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn InsertColor(
        &self,
        propertyname: &windows_core::HSTRING,
        value: Color,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).InsertColor)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertMatrix3x2(
        &self,
        propertyname: &windows_core::HSTRING,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).InsertMatrix3x2)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertMatrix4x4(
        &self,
        propertyname: &windows_core::HSTRING,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).InsertMatrix4x4)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertQuaternion(
        &self,
        propertyname: &windows_core::HSTRING,
        value: Quaternion,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).InsertQuaternion)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertScalar(
        &self,
        propertyname: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).InsertScalar)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertVector2(
        &self,
        propertyname: &windows_core::HSTRING,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).InsertVector2)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertVector3(
        &self,
        propertyname: &windows_core::HSTRING,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).InsertVector3)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertVector4(
        &self,
        propertyname: &windows_core::HSTRING,
        value: windows_numerics::Vector4,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).InsertVector4)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn TryGetColor(
        &self,
        propertyname: &windows_core::HSTRING,
        value: &mut Color,
    ) -> windows_core::Result<CompositionGetValueStatus> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TryGetColor)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                value,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn TryGetMatrix3x2(
        &self,
        propertyname: &windows_core::HSTRING,
        value: &mut windows_numerics::Matrix3x2,
    ) -> windows_core::Result<CompositionGetValueStatus> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TryGetMatrix3x2)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                value,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn TryGetMatrix4x4(
        &self,
        propertyname: &windows_core::HSTRING,
        value: &mut windows_numerics::Matrix4x4,
    ) -> windows_core::Result<CompositionGetValueStatus> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TryGetMatrix4x4)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                value,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn TryGetQuaternion(
        &self,
        propertyname: &windows_core::HSTRING,
        value: &mut Quaternion,
    ) -> windows_core::Result<CompositionGetValueStatus> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TryGetQuaternion)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                value,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn TryGetScalar(
        &self,
        propertyname: &windows_core::HSTRING,
        value: &mut f32,
    ) -> windows_core::Result<CompositionGetValueStatus> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TryGetScalar)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                value,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn TryGetVector2(
        &self,
        propertyname: &windows_core::HSTRING,
        value: &mut windows_numerics::Vector2,
    ) -> windows_core::Result<CompositionGetValueStatus> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TryGetVector2)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                value,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn TryGetVector3(
        &self,
        propertyname: &windows_core::HSTRING,
        value: &mut windows_numerics::Vector3,
    ) -> windows_core::Result<CompositionGetValueStatus> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TryGetVector3)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                value,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn TryGetVector4(
        &self,
        propertyname: &windows_core::HSTRING,
        value: &mut windows_numerics::Vector4,
    ) -> windows_core::Result<CompositionGetValueStatus> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TryGetVector4)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(propertyname),
                value,
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
impl windows_core::RuntimeType for CompositionPropertySet {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionPropertySet>();
}
unsafe impl windows_core::Interface for CompositionPropertySet {
    type Vtable = <ICompositionPropertySet as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionPropertySet as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionPropertySet {
    const NAME: &'static str = "Windows.UI.Composition.CompositionPropertySet";
}
unsafe impl Send for CompositionPropertySet {}
unsafe impl Sync for CompositionPropertySet {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionRoundedRectangleGeometry(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionRoundedRectangleGeometry,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    CompositionRoundedRectangleGeometry,
    IAnimationObject,
    CompositionGeometry,
    CompositionObject
);
impl CompositionRoundedRectangleGeometry {
    pub(crate) fn TrimEnd(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<ICompositionGeometry>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).TrimEnd)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetTrimEnd(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionGeometry>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTrimEnd)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn TrimOffset(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<ICompositionGeometry>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).TrimOffset)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetTrimOffset(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionGeometry>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTrimOffset)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn TrimStart(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<ICompositionGeometry>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).TrimStart)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetTrimStart(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionGeometry>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTrimStart)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn CornerRadius(&self) -> windows_core::Result<windows_numerics::Vector2> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CornerRadius)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetCornerRadius(
        &self,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetCornerRadius)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Offset(&self) -> windows_core::Result<windows_numerics::Vector2> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Offset)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetOffset(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetOffset)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Size(&self) -> windows_core::Result<windows_numerics::Vector2> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Size)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetSize(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetSize)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionRoundedRectangleGeometry {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionRoundedRectangleGeometry>();
}
unsafe impl windows_core::Interface for CompositionRoundedRectangleGeometry {
    type Vtable = <ICompositionRoundedRectangleGeometry as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <ICompositionRoundedRectangleGeometry as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionRoundedRectangleGeometry {
    const NAME: &'static str = "Windows.UI.Composition.CompositionRoundedRectangleGeometry";
}
unsafe impl Send for CompositionRoundedRectangleGeometry {}
unsafe impl Sync for CompositionRoundedRectangleGeometry {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionScopedBatch(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionScopedBatch,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionScopedBatch, IAnimationObject, CompositionObject);
impl CompositionScopedBatch {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn IsActive(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsActive)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn IsEnded(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsEnded)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn End(&self) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).End)(windows_core::Interface::as_raw(self)).ok()
        }
    }
    pub(crate) fn Resume(&self) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).Resume)(windows_core::Interface::as_raw(self))
                .ok()
        }
    }
    pub(crate) fn Suspend(&self) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).Suspend)(windows_core::Interface::as_raw(self))
                .ok()
        }
    }
    pub(crate) fn Completed<F>(
        &self,
        handler: F,
    ) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(
                windows_core::Ref<windows_core::IInspectable>,
                windows_core::Ref<CompositionBatchCompletedEventArgs>,
            ) + Send
            + 'static,
    {
        let handler = <TypedEventHandler<
            windows_core::IInspectable,
            CompositionBatchCompletedEventArgs,
        >>::new(move |a0, a1| {
            handler(a0, a1);
            Ok(())
        });
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).Completed)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveCompleted,
            ))
        }
    }
}
impl windows_core::RuntimeType for CompositionScopedBatch {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionScopedBatch>();
}
unsafe impl windows_core::Interface for CompositionScopedBatch {
    type Vtable = <ICompositionScopedBatch as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionScopedBatch as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionScopedBatch {
    const NAME: &'static str = "Windows.UI.Composition.CompositionScopedBatch";
}
unsafe impl Send for CompositionScopedBatch {}
unsafe impl Sync for CompositionScopedBatch {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionShadow(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionShadow,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionShadow, CompositionObject);
impl CompositionShadow {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionShadow {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionShadow>();
}
unsafe impl windows_core::Interface for CompositionShadow {
    type Vtable = <ICompositionShadow as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionShadow as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionShadow {
    const NAME: &'static str = "Windows.UI.Composition.CompositionShadow";
}
unsafe impl Send for CompositionShadow {}
unsafe impl Sync for CompositionShadow {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionSurfaceBrush(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionSurfaceBrush,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    CompositionSurfaceBrush,
    CompositionBrush,
    CompositionObject
);
impl CompositionSurfaceBrush {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionSurfaceBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionSurfaceBrush>();
}
unsafe impl windows_core::Interface for CompositionSurfaceBrush {
    type Vtable = <ICompositionSurfaceBrush as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionSurfaceBrush as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionSurfaceBrush {
    const NAME: &'static str = "Windows.UI.Composition.CompositionSurfaceBrush";
}
unsafe impl Send for CompositionSurfaceBrush {}
unsafe impl Sync for CompositionSurfaceBrush {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionTarget(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionTarget,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionTarget, CompositionObject);
impl CompositionTarget {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionTarget {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionTarget>();
}
unsafe impl windows_core::Interface for CompositionTarget {
    type Vtable = <ICompositionTarget as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionTarget as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionTarget {
    const NAME: &'static str = "Windows.UI.Composition.CompositionTarget";
}
unsafe impl Send for CompositionTarget {}
unsafe impl Sync for CompositionTarget {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionViewBox(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionViewBox,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionViewBox, CompositionObject);
impl CompositionViewBox {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionViewBox {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionViewBox>();
}
unsafe impl windows_core::Interface for CompositionViewBox {
    type Vtable = <ICompositionViewBox as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionViewBox as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionViewBox {
    const NAME: &'static str = "Windows.UI.Composition.CompositionViewBox";
}
unsafe impl Send for CompositionViewBox {}
unsafe impl Sync for CompositionViewBox {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Compositor(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    Compositor,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl Compositor {
    pub(crate) fn new() -> windows_core::Result<Self> {
        Self::IActivationFactory(|f| f.ActivateInstance::<Self>())
    }
    fn IActivationFactory<
        R,
        F: FnOnce(&windows_core::imp::IGenericFactory) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            Compositor,
            windows_core::imp::IGenericFactory,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
    pub(crate) fn CreateColorKeyFrameAnimation(
        &self,
    ) -> windows_core::Result<ColorKeyFrameAnimation> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateColorKeyFrameAnimation)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateColorBrush(&self) -> windows_core::Result<CompositionColorBrush> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateColorBrush)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateColorBrushWithColor(
        &self,
        color: Color,
    ) -> windows_core::Result<CompositionColorBrush> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateColorBrushWithColor)(
                windows_core::Interface::as_raw(self),
                color,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateContainerVisual(&self) -> windows_core::Result<ContainerVisual> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateContainerVisual)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateCubicBezierEasingFunction(
        &self,
        controlpoint1: windows_numerics::Vector2,
        controlpoint2: windows_numerics::Vector2,
    ) -> windows_core::Result<CubicBezierEasingFunction> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateCubicBezierEasingFunction)(
                windows_core::Interface::as_raw(self),
                controlpoint1,
                controlpoint2,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateEffectFactory<P0>(
        &self,
        graphicseffect: P0,
    ) -> windows_core::Result<CompositionEffectFactory>
    where
        P0: windows_core::Param<IGraphicsEffect>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateEffectFactory)(
                windows_core::Interface::as_raw(self),
                graphicseffect.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateEffectFactoryWithProperties<P0, P1>(
        &self,
        graphicseffect: P0,
        animatableproperties: P1,
    ) -> windows_core::Result<CompositionEffectFactory>
    where
        P0: windows_core::Param<IGraphicsEffect>,
        P1: windows_core::Param<IIterable<windows_core::HSTRING>>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateEffectFactoryWithProperties)(
                windows_core::Interface::as_raw(self),
                graphicseffect.param().abi(),
                animatableproperties.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateExpressionAnimation(&self) -> windows_core::Result<ExpressionAnimation> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateExpressionAnimation)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateExpressionAnimationWithExpression(
        &self,
        expression: &windows_core::HSTRING,
    ) -> windows_core::Result<ExpressionAnimation> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateExpressionAnimationWithExpression)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(expression),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateInsetClip(&self) -> windows_core::Result<InsetClip> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateInsetClip)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateInsetClipWithInsets(
        &self,
        leftinset: f32,
        topinset: f32,
        rightinset: f32,
        bottominset: f32,
    ) -> windows_core::Result<InsetClip> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateInsetClipWithInsets)(
                windows_core::Interface::as_raw(self),
                leftinset,
                topinset,
                rightinset,
                bottominset,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateLinearEasingFunction(&self) -> windows_core::Result<LinearEasingFunction> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateLinearEasingFunction)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreatePropertySet(&self) -> windows_core::Result<CompositionPropertySet> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreatePropertySet)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateQuaternionKeyFrameAnimation(
        &self,
    ) -> windows_core::Result<QuaternionKeyFrameAnimation> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateQuaternionKeyFrameAnimation)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateScalarKeyFrameAnimation(
        &self,
    ) -> windows_core::Result<ScalarKeyFrameAnimation> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateScalarKeyFrameAnimation)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateScopedBatch(
        &self,
        batchtype: CompositionBatchTypes,
    ) -> windows_core::Result<CompositionScopedBatch> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateScopedBatch)(
                windows_core::Interface::as_raw(self),
                batchtype,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateSpriteVisual(&self) -> windows_core::Result<SpriteVisual> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateSpriteVisual)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateSurfaceBrush(&self) -> windows_core::Result<CompositionSurfaceBrush> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateSurfaceBrush)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateSurfaceBrushWithSurface<P0>(
        &self,
        surface: P0,
    ) -> windows_core::Result<CompositionSurfaceBrush>
    where
        P0: windows_core::Param<ICompositionSurface>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateSurfaceBrushWithSurface)(
                windows_core::Interface::as_raw(self),
                surface.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateTargetForCurrentView(&self) -> windows_core::Result<CompositionTarget> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateTargetForCurrentView)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateVector2KeyFrameAnimation(
        &self,
    ) -> windows_core::Result<Vector2KeyFrameAnimation> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateVector2KeyFrameAnimation)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateVector3KeyFrameAnimation(
        &self,
    ) -> windows_core::Result<Vector3KeyFrameAnimation> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateVector3KeyFrameAnimation)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateVector4KeyFrameAnimation(
        &self,
    ) -> windows_core::Result<Vector4KeyFrameAnimation> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateVector4KeyFrameAnimation)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn GetCommitBatch(
        &self,
        batchtype: CompositionBatchTypes,
    ) -> windows_core::Result<CompositionCommitBatch> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCommitBatch)(
                windows_core::Interface::as_raw(self),
                batchtype,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateAmbientLight(&self) -> windows_core::Result<AmbientLight> {
        let this = &windows_core::Interface::cast::<ICompositor2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateAmbientLight)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateAnimationGroup(&self) -> windows_core::Result<CompositionAnimationGroup> {
        let this = &windows_core::Interface::cast::<ICompositor2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateAnimationGroup)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateBackdropBrush(&self) -> windows_core::Result<CompositionBackdropBrush> {
        let this = &windows_core::Interface::cast::<ICompositor2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateBackdropBrush)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateDistantLight(&self) -> windows_core::Result<DistantLight> {
        let this = &windows_core::Interface::cast::<ICompositor2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateDistantLight)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateDropShadow(&self) -> windows_core::Result<DropShadow> {
        let this = &windows_core::Interface::cast::<ICompositor2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateDropShadow)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateImplicitAnimationCollection(
        &self,
    ) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositor2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateImplicitAnimationCollection)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateLayerVisual(&self) -> windows_core::Result<LayerVisual> {
        let this = &windows_core::Interface::cast::<ICompositor2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateLayerVisual)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateMaskBrush(&self) -> windows_core::Result<CompositionMaskBrush> {
        let this = &windows_core::Interface::cast::<ICompositor2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateMaskBrush)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateNineGridBrush(&self) -> windows_core::Result<CompositionNineGridBrush> {
        let this = &windows_core::Interface::cast::<ICompositor2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateNineGridBrush)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreatePointLight(&self) -> windows_core::Result<PointLight> {
        let this = &windows_core::Interface::cast::<ICompositor2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreatePointLight)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateSpotLight(&self) -> windows_core::Result<SpotLight> {
        let this = &windows_core::Interface::cast::<ICompositor2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateSpotLight)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateStepEasingFunction(&self) -> windows_core::Result<StepEasingFunction> {
        let this = &windows_core::Interface::cast::<ICompositor2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateStepEasingFunction)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateStepEasingFunctionWithStepCount(
        &self,
        stepcount: i32,
    ) -> windows_core::Result<StepEasingFunction> {
        let this = &windows_core::Interface::cast::<ICompositor2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateStepEasingFunctionWithStepCount)(
                windows_core::Interface::as_raw(this),
                stepcount,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateColorGradientStop(
        &self,
    ) -> windows_core::Result<CompositionColorGradientStop> {
        let this = &windows_core::Interface::cast::<ICompositor4>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateColorGradientStop)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateColorGradientStopWithOffsetAndColor(
        &self,
        offset: f32,
        color: Color,
    ) -> windows_core::Result<CompositionColorGradientStop> {
        let this = &windows_core::Interface::cast::<ICompositor4>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateColorGradientStopWithOffsetAndColor)(
                windows_core::Interface::as_raw(this),
                offset,
                color,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateLinearGradientBrush(
        &self,
    ) -> windows_core::Result<CompositionLinearGradientBrush> {
        let this = &windows_core::Interface::cast::<ICompositor4>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateLinearGradientBrush)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateSpringScalarAnimation(
        &self,
    ) -> windows_core::Result<SpringScalarNaturalMotionAnimation> {
        let this = &windows_core::Interface::cast::<ICompositor4>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateSpringScalarAnimation)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateSpringVector2Animation(
        &self,
    ) -> windows_core::Result<SpringVector2NaturalMotionAnimation> {
        let this = &windows_core::Interface::cast::<ICompositor4>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateSpringVector2Animation)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateSpringVector3Animation(
        &self,
    ) -> windows_core::Result<SpringVector3NaturalMotionAnimation> {
        let this = &windows_core::Interface::cast::<ICompositor4>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateSpringVector3Animation)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateRoundedRectangleGeometry(
        &self,
    ) -> windows_core::Result<CompositionRoundedRectangleGeometry> {
        let this = &windows_core::Interface::cast::<ICompositor5>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateRoundedRectangleGeometry)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateGeometricClipWithGeometry<P0>(
        &self,
        geometry: P0,
    ) -> windows_core::Result<CompositionGeometricClip>
    where
        P0: windows_core::Param<CompositionGeometry>,
    {
        let this = &windows_core::Interface::cast::<ICompositor6>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateGeometricClipWithGeometry)(
                windows_core::Interface::as_raw(this),
                geometry.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn MaxGlobalPlaybackRate() -> windows_core::Result<f32> {
        Self::ICompositorStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).MaxGlobalPlaybackRate)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        })
    }
    pub(crate) fn MinGlobalPlaybackRate() -> windows_core::Result<f32> {
        Self::ICompositorStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).MinGlobalPlaybackRate)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        })
    }
    fn ICompositorStatics<R, F: FnOnce(&ICompositorStatics) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<Compositor, ICompositorStatics> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for Compositor {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositor>();
}
unsafe impl windows_core::Interface for Compositor {
    type Vtable = <ICompositor as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositor as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for Compositor {
    const NAME: &'static str = "Windows.UI.Composition.Compositor";
}
unsafe impl Send for Compositor {}
unsafe impl Sync for Compositor {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContainerVisual(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ContainerVisual,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    ContainerVisual,
    IAnimationObject,
    Visual,
    CompositionObject
);
impl ContainerVisual {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn Children(&self) -> windows_core::Result<VisualCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Children)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn AnchorPoint(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).AnchorPoint)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetAnchorPoint(
        &self,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetAnchorPoint)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn BackfaceVisibility(&self) -> windows_core::Result<CompositionBackfaceVisibility> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).BackfaceVisibility)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetBackfaceVisibility(
        &self,
        value: CompositionBackfaceVisibility,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBackfaceVisibility)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn BorderMode(&self) -> windows_core::Result<CompositionBorderMode> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).BorderMode)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetBorderMode(&self, value: CompositionBorderMode) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBorderMode)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn CenterPoint(&self) -> windows_core::Result<windows_numerics::Vector3> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CenterPoint)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetCenterPoint(
        &self,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetCenterPoint)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Clip(&self) -> windows_core::Result<CompositionClip> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Clip)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetClip<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<CompositionClip>,
    {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetClip)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn CompositeMode(&self) -> windows_core::Result<CompositionCompositeMode> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CompositeMode)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetCompositeMode(
        &self,
        value: CompositionCompositeMode,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetCompositeMode)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IsVisible(&self) -> windows_core::Result<bool> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IsVisible)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIsVisible(&self, value: bool) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIsVisible)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Offset(&self) -> windows_core::Result<windows_numerics::Vector3> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Offset)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetOffset(&self, value: windows_numerics::Vector3) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetOffset)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Opacity(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Opacity)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetOpacity(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetOpacity)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Orientation(&self) -> windows_core::Result<Quaternion> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Orientation)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetOrientation(&self, value: Quaternion) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetOrientation)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Parent(&self) -> windows_core::Result<Self> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Parent)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn RotationAngle(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RotationAngle)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAngle(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRotationAngle)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RotationAngleInDegrees(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RotationAngleInDegrees)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAngleInDegrees(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRotationAngleInDegrees)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RotationAxis(&self) -> windows_core::Result<windows_numerics::Vector3> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RotationAxis)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAxis(
        &self,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRotationAxis)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Scale(&self) -> windows_core::Result<windows_numerics::Vector3> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Scale)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetScale(&self, value: windows_numerics::Vector3) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScale)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Size(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Size)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetSize(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetSize)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn TransformMatrix(&self) -> windows_core::Result<windows_numerics::Matrix4x4> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).TransformMatrix)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetTransformMatrix(
        &self,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTransformMatrix)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn ParentForTransform(&self) -> windows_core::Result<Visual> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ParentForTransform)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetParentForTransform<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Visual>,
    {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetParentForTransform)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn RelativeOffsetAdjustment(
        &self,
    ) -> windows_core::Result<windows_numerics::Vector3> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RelativeOffsetAdjustment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRelativeOffsetAdjustment(
        &self,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRelativeOffsetAdjustment)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RelativeSizeAdjustment(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RelativeSizeAdjustment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRelativeSizeAdjustment(
        &self,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRelativeSizeAdjustment)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for ContainerVisual {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IContainerVisual>();
}
unsafe impl windows_core::Interface for ContainerVisual {
    type Vtable = <IContainerVisual as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IContainerVisual as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for ContainerVisual {
    const NAME: &'static str = "Windows.UI.Composition.ContainerVisual";
}
unsafe impl Send for ContainerVisual {}
unsafe impl Sync for ContainerVisual {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreDispatcher(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CoreDispatcher,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for CoreDispatcher {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICoreDispatcher>();
}
unsafe impl windows_core::Interface for CoreDispatcher {
    type Vtable = <ICoreDispatcher as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICoreDispatcher as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CoreDispatcher {
    const NAME: &'static str = "Windows.UI.Core.CoreDispatcher";
}
unsafe impl Send for CoreDispatcher {}
unsafe impl Sync for CoreDispatcher {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CubicBezierEasingFunction(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CubicBezierEasingFunction,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    CubicBezierEasingFunction,
    IAnimationObject,
    CompositionEasingFunction,
    CompositionObject
);
impl CubicBezierEasingFunction {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for CubicBezierEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICubicBezierEasingFunction>();
}
unsafe impl windows_core::Interface for CubicBezierEasingFunction {
    type Vtable = <ICubicBezierEasingFunction as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICubicBezierEasingFunction as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CubicBezierEasingFunction {
    const NAME: &'static str = "Windows.UI.Composition.CubicBezierEasingFunction";
}
unsafe impl Send for CubicBezierEasingFunction {}
unsafe impl Sync for CubicBezierEasingFunction {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DistantLight(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    DistantLight,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(DistantLight, CompositionLight, CompositionObject);
impl DistantLight {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for DistantLight {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDistantLight>();
}
unsafe impl windows_core::Interface for DistantLight {
    type Vtable = <IDistantLight as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDistantLight as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for DistantLight {
    const NAME: &'static str = "Windows.UI.Composition.DistantLight";
}
unsafe impl Send for DistantLight {}
unsafe impl Sync for DistantLight {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DropShadow(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    DropShadow,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(DropShadow, CompositionShadow, CompositionObject);
impl DropShadow {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for DropShadow {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDropShadow>();
}
unsafe impl windows_core::Interface for DropShadow {
    type Vtable = <IDropShadow as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDropShadow as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for DropShadow {
    const NAME: &'static str = "Windows.UI.Composition.DropShadow";
}
unsafe impl Send for DropShadow {}
unsafe impl Sync for DropShadow {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ElasticEasingFunction(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ElasticEasingFunction,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    ElasticEasingFunction,
    CompositionEasingFunction,
    CompositionObject
);
impl ElasticEasingFunction {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for ElasticEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IElasticEasingFunction>();
}
unsafe impl windows_core::Interface for ElasticEasingFunction {
    type Vtable = <IElasticEasingFunction as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IElasticEasingFunction as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for ElasticEasingFunction {
    const NAME: &'static str = "Windows.UI.Composition.ElasticEasingFunction";
}
unsafe impl Send for ElasticEasingFunction {}
unsafe impl Sync for ElasticEasingFunction {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExponentialEasingFunction(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ExponentialEasingFunction,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    ExponentialEasingFunction,
    CompositionEasingFunction,
    CompositionObject
);
impl ExponentialEasingFunction {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for ExponentialEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IExponentialEasingFunction>();
}
unsafe impl windows_core::Interface for ExponentialEasingFunction {
    type Vtable = <IExponentialEasingFunction as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IExponentialEasingFunction as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for ExponentialEasingFunction {
    const NAME: &'static str = "Windows.UI.Composition.ExponentialEasingFunction";
}
unsafe impl Send for ExponentialEasingFunction {}
unsafe impl Sync for ExponentialEasingFunction {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpressionAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ExpressionAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    ExpressionAnimation,
    IAnimationObject,
    ICompositionAnimationBase,
    CompositionAnimation,
    CompositionObject
);
impl ExpressionAnimation {
    pub(crate) fn ClearAllParameters(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearAllParameters)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub(crate) fn ClearParameter(&self, key: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
            )
            .ok()
        }
    }
    pub(crate) fn SetColorParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Color,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetColorParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix3x2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix3x2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix4x4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix4x4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetQuaternionParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Quaternion,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetQuaternionParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetReferenceParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScalarParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector3Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector3Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetBooleanParameter(
        &self,
        key: &windows_core::HSTRING,
        value: bool,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBooleanParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Target(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Target)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetTarget(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTarget)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn Expression(&self) -> windows_core::Result<windows_core::HSTRING> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Expression)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetExpression(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetExpression)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for ExpressionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IExpressionAnimation>();
}
unsafe impl windows_core::Interface for ExpressionAnimation {
    type Vtable = <IExpressionAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IExpressionAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for ExpressionAnimation {
    const NAME: &'static str = "Windows.UI.Composition.ExpressionAnimation";
}
unsafe impl Send for ExpressionAnimation {}
unsafe impl Sync for ExpressionAnimation {}
windows_core::imp::define_interface!(
    IAmbientLight,
    IAmbientLight_Vtbl,
    0xa48130a1_b7c4_46f7_b9bf_daf43a44e6ee
);
impl windows_core::RuntimeType for IAmbientLight {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IAmbientLight");
}
#[repr(C)]
pub struct IAmbientLight_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IAnimationObject,
    IAnimationObject_Vtbl,
    0xe7141e0a_04b8_4fc5_a4dc_195392e57807
);
impl windows_core::RuntimeType for IAnimationObject {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IAnimationObject");
}
windows_core::imp::interface_hierarchy!(
    IAnimationObject,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeName for IAnimationObject {
    const NAME: &'static str = "Windows.UI.Composition.IAnimationObject";
}
#[repr(C)]
pub struct IAnimationObject_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IBackEasingFunction,
    IBackEasingFunction_Vtbl,
    0xb8560da4_5e3c_545d_b263_7987a2bd27cb
);
impl windows_core::RuntimeType for IBackEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IBackEasingFunction");
}
#[repr(C)]
pub struct IBackEasingFunction_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IBounceEasingFunction,
    IBounceEasingFunction_Vtbl,
    0xe7fdb44b_aad5_5174_9421_eef8b75a6a43
);
impl windows_core::RuntimeType for IBounceEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IBounceEasingFunction");
}
#[repr(C)]
pub struct IBounceEasingFunction_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICircleEasingFunction,
    ICircleEasingFunction_Vtbl,
    0x1e07222a_6f82_5a28_8748_2e92fc46ee2b
);
impl windows_core::RuntimeType for ICircleEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICircleEasingFunction");
}
#[repr(C)]
pub struct ICircleEasingFunction_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IColorKeyFrameAnimation,
    IColorKeyFrameAnimation_Vtbl,
    0x93adb5e9_8e05_4593_84a3_dca152781e56
);
impl windows_core::RuntimeType for IColorKeyFrameAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.IColorKeyFrameAnimation",
    );
}
#[repr(C)]
pub struct IColorKeyFrameAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub InterpolationColorSpace: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut CompositionColorSpace,
    ) -> windows_core::HRESULT,
    pub SetInterpolationColorSpace: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        CompositionColorSpace,
    ) -> windows_core::HRESULT,
    pub InsertKeyFrame:
        unsafe extern "system" fn(*mut core::ffi::c_void, f32, Color) -> windows_core::HRESULT,
    pub InsertKeyFrameWithEasingFunction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        f32,
        Color,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionAnimation,
    ICompositionAnimation_Vtbl,
    0x464c4c2c_1caa_4061_9b40_e13fde1503ca
);
impl windows_core::RuntimeType for ICompositionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositionAnimation");
}
#[repr(C)]
pub struct ICompositionAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub ClearAllParameters:
        unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ClearParameter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetColorParameter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        Color,
    ) -> windows_core::HRESULT,
    pub SetMatrix3x2Parameter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        windows_numerics::Matrix3x2,
    ) -> windows_core::HRESULT,
    pub SetMatrix4x4Parameter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        windows_numerics::Matrix4x4,
    ) -> windows_core::HRESULT,
    pub SetQuaternionParameter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        Quaternion,
    ) -> windows_core::HRESULT,
    pub SetReferenceParameter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetScalarParameter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        f32,
    ) -> windows_core::HRESULT,
    pub SetVector2Parameter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub SetVector3Parameter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        windows_numerics::Vector3,
    ) -> windows_core::HRESULT,
    pub SetVector4Parameter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        windows_numerics::Vector4,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionAnimation2,
    ICompositionAnimation2_Vtbl,
    0x369b603e_a80f_4948_93e3_ed23fb38c6cb
);
impl windows_core::RuntimeType for ICompositionAnimation2 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionAnimation2",
    );
}
#[repr(C)]
pub struct ICompositionAnimation2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub SetBooleanParameter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        bool,
    ) -> windows_core::HRESULT,
    pub Target: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetTarget: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionAnimationBase,
    ICompositionAnimationBase_Vtbl,
    0x1c2c2999_e818_48d3_a6dd_d78c82f8ace9
);
impl windows_core::RuntimeType for ICompositionAnimationBase {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionAnimationBase",
    );
}
windows_core::imp::interface_hierarchy!(
    ICompositionAnimationBase,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeName for ICompositionAnimationBase {
    const NAME: &'static str = "Windows.UI.Composition.ICompositionAnimationBase";
}
pub trait ICompositionAnimationBase_Impl: windows_core::IUnknownImpl {}
impl ICompositionAnimationBase_Vtbl {
    pub const fn new<Identity: ICompositionAnimationBase_Impl, const OFFSET: isize>() -> Self {
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<
                Identity,
                ICompositionAnimationBase,
                OFFSET,
            >(),
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<ICompositionAnimationBase as windows_core::Interface>::IID
    }
}
#[repr(C)]
pub struct ICompositionAnimationBase_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionAnimationGroup,
    ICompositionAnimationGroup_Vtbl,
    0x5e7cc90c_cd14_4e07_8a55_c72527aabdac
);
impl windows_core::RuntimeType for ICompositionAnimationGroup {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionAnimationGroup",
    );
}
#[repr(C)]
pub struct ICompositionAnimationGroup_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionBackdropBrush,
    ICompositionBackdropBrush_Vtbl,
    0xc5acae58_3898_499e_8d7f_224e91286a5d
);
impl windows_core::RuntimeType for ICompositionBackdropBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionBackdropBrush",
    );
}
#[repr(C)]
pub struct ICompositionBackdropBrush_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionBatchCompletedEventArgs,
    ICompositionBatchCompletedEventArgs_Vtbl,
    0x0d00dad0_9464_450a_a562_2e2698b0a812
);
impl windows_core::RuntimeType for ICompositionBatchCompletedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionBatchCompletedEventArgs",
    );
}
#[repr(C)]
pub struct ICompositionBatchCompletedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionBrush,
    ICompositionBrush_Vtbl,
    0xab0d7608_30c0_40e9_b568_b60a6bd1fb46
);
impl windows_core::RuntimeType for ICompositionBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositionBrush");
}
#[repr(C)]
pub struct ICompositionBrush_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionClip,
    ICompositionClip_Vtbl,
    0x1ccd2a52_cfc7_4ace_9983_146bb8eb6a3c
);
impl windows_core::RuntimeType for ICompositionClip {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositionClip");
}
#[repr(C)]
pub struct ICompositionClip_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionClip2,
    ICompositionClip2_Vtbl,
    0x5893e069_3516_40e1_89e0_5ba924927235
);
impl windows_core::RuntimeType for ICompositionClip2 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositionClip2");
}
#[repr(C)]
pub struct ICompositionClip2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub AnchorPoint: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub SetAnchorPoint: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub CenterPoint: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub SetCenterPoint: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub Offset: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub SetOffset: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub RotationAngle:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32) -> windows_core::HRESULT,
    pub SetRotationAngle:
        unsafe extern "system" fn(*mut core::ffi::c_void, f32) -> windows_core::HRESULT,
    pub RotationAngleInDegrees:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32) -> windows_core::HRESULT,
    pub SetRotationAngleInDegrees:
        unsafe extern "system" fn(*mut core::ffi::c_void, f32) -> windows_core::HRESULT,
    pub Scale: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub SetScale: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub TransformMatrix: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Matrix3x2,
    ) -> windows_core::HRESULT,
    pub SetTransformMatrix: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Matrix3x2,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionColorBrush,
    ICompositionColorBrush_Vtbl,
    0x2b264c5e_bf35_4831_8642_cf70c20fff2f
);
impl windows_core::RuntimeType for ICompositionColorBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionColorBrush",
    );
}
#[repr(C)]
pub struct ICompositionColorBrush_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Color:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut Color) -> windows_core::HRESULT,
    pub SetColor: unsafe extern "system" fn(*mut core::ffi::c_void, Color) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionColorGradientStop,
    ICompositionColorGradientStop_Vtbl,
    0x6f00ca92_c801_4e41_9a8f_a53e20f57778
);
impl windows_core::RuntimeType for ICompositionColorGradientStop {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionColorGradientStop",
    );
}
#[repr(C)]
pub struct ICompositionColorGradientStop_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionCommitBatch,
    ICompositionCommitBatch_Vtbl,
    0x0d00dad0_ca07_4400_8c8e_cb5db08559cc
);
impl windows_core::RuntimeType for ICompositionCommitBatch {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionCommitBatch",
    );
}
#[repr(C)]
pub struct ICompositionCommitBatch_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionEasingFunction,
    ICompositionEasingFunction_Vtbl,
    0x5145e356_bf79_4ea8_8cc2_6b5b472e6c9a
);
impl windows_core::RuntimeType for ICompositionEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionEasingFunction",
    );
}
#[repr(C)]
pub struct ICompositionEasingFunction_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionEasingFunctionStatics,
    ICompositionEasingFunctionStatics_Vtbl,
    0x17a766b6_2936_53ea_b5af_c642f4a61083
);
impl windows_core::RuntimeType for ICompositionEasingFunctionStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionEasingFunctionStatics",
    );
}
#[repr(C)]
pub struct ICompositionEasingFunctionStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateCubicBezierEasingFunction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
        windows_numerics::Vector2,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateLinearEasingFunction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateStepEasingFunction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateStepEasingFunctionWithStepCount: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        i32,
        *mut *mut core::ffi::c_void,
    )
        -> windows_core::HRESULT,
    pub CreateBackEasingFunction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        CompositionEasingFunctionMode,
        f32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBounceEasingFunction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        CompositionEasingFunctionMode,
        i32,
        f32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateCircleEasingFunction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        CompositionEasingFunctionMode,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateElasticEasingFunction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        CompositionEasingFunctionMode,
        i32,
        f32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateExponentialEasingFunction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        CompositionEasingFunctionMode,
        f32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreatePowerEasingFunction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        CompositionEasingFunctionMode,
        f32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateSineEasingFunction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        CompositionEasingFunctionMode,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionEffectFactory,
    ICompositionEffectFactory_Vtbl,
    0xbe5624af_ba7e_4510_9850_41c0b4ff74df
);
impl windows_core::RuntimeType for ICompositionEffectFactory {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionEffectFactory",
    );
}
#[repr(C)]
pub struct ICompositionEffectFactory_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionGeometricClip,
    ICompositionGeometricClip_Vtbl,
    0xc840b581_81c9_4444_a2c1_ccaece3a50e5
);
impl windows_core::RuntimeType for ICompositionGeometricClip {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionGeometricClip",
    );
}
#[repr(C)]
pub struct ICompositionGeometricClip_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Geometry: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetGeometry: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ViewBox: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetViewBox: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionGeometry,
    ICompositionGeometry_Vtbl,
    0xe985217c_6a17_4207_abd8_5fd3dd612a9d
);
impl windows_core::RuntimeType for ICompositionGeometry {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositionGeometry");
}
#[repr(C)]
pub struct ICompositionGeometry_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub TrimEnd:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32) -> windows_core::HRESULT,
    pub SetTrimEnd: unsafe extern "system" fn(*mut core::ffi::c_void, f32) -> windows_core::HRESULT,
    pub TrimOffset:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32) -> windows_core::HRESULT,
    pub SetTrimOffset:
        unsafe extern "system" fn(*mut core::ffi::c_void, f32) -> windows_core::HRESULT,
    pub TrimStart:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32) -> windows_core::HRESULT,
    pub SetTrimStart:
        unsafe extern "system" fn(*mut core::ffi::c_void, f32) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionGradientBrush,
    ICompositionGradientBrush_Vtbl,
    0x1d9709e0_ffc6_4c0e_a9ab_34144d4c9098
);
impl windows_core::RuntimeType for ICompositionGradientBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionGradientBrush",
    );
}
#[repr(C)]
pub struct ICompositionGradientBrush_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionLight,
    ICompositionLight_Vtbl,
    0x41a6d7c2_2e5d_4bc1_b09e_8f0a03e3d8d3
);
impl windows_core::RuntimeType for ICompositionLight {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositionLight");
}
#[repr(C)]
pub struct ICompositionLight_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionLinearGradientBrush,
    ICompositionLinearGradientBrush_Vtbl,
    0x983bc519_a9db_413c_a2d8_2a9056fc525e
);
impl windows_core::RuntimeType for ICompositionLinearGradientBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionLinearGradientBrush",
    );
}
#[repr(C)]
pub struct ICompositionLinearGradientBrush_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionMaskBrush,
    ICompositionMaskBrush_Vtbl,
    0x522cf09e_be6b_4f41_be49_f9226d471b4a
);
impl windows_core::RuntimeType for ICompositionMaskBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositionMaskBrush");
}
#[repr(C)]
pub struct ICompositionMaskBrush_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionNineGridBrush,
    ICompositionNineGridBrush_Vtbl,
    0xf25154e4_bc8c_4be7_b80f_8685b83c0186
);
impl windows_core::RuntimeType for ICompositionNineGridBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionNineGridBrush",
    );
}
#[repr(C)]
pub struct ICompositionNineGridBrush_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionObject,
    ICompositionObject_Vtbl,
    0xbcb4ad45_7609_4550_934f_16002a68fded
);
impl windows_core::RuntimeType for ICompositionObject {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositionObject");
}
#[repr(C)]
pub struct ICompositionObject_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Compositor: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub Dispatcher: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub Properties: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub StartAnimation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub StopAnimation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionObject2,
    ICompositionObject2_Vtbl,
    0xef874ea1_5cff_4b68_9e30_a1519d08ba03
);
impl windows_core::RuntimeType for ICompositionObject2 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositionObject2");
}
#[repr(C)]
pub struct ICompositionObject2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Comment: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetComment: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ImplicitAnimations: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetImplicitAnimations: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub StartAnimationGroup: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub StopAnimationGroup: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionObjectStatics,
    ICompositionObjectStatics_Vtbl,
    0xc1ed052f_1ba2_44ba_a904_6a882a0a5adb
);
impl windows_core::RuntimeType for ICompositionObjectStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionObjectStatics",
    );
}
#[repr(C)]
pub struct ICompositionObjectStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub StartAnimationWithIAnimationObject: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub StartAnimationGroupWithIAnimationObject: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    )
        -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionPropertySet,
    ICompositionPropertySet_Vtbl,
    0xc9d6d202_5f67_4453_9117_9eadd430d3c2
);
impl windows_core::RuntimeType for ICompositionPropertySet {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionPropertySet",
    );
}
#[repr(C)]
pub struct ICompositionPropertySet_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub InsertColor: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        Color,
    ) -> windows_core::HRESULT,
    pub InsertMatrix3x2: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        windows_numerics::Matrix3x2,
    ) -> windows_core::HRESULT,
    pub InsertMatrix4x4: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        windows_numerics::Matrix4x4,
    ) -> windows_core::HRESULT,
    pub InsertQuaternion: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        Quaternion,
    ) -> windows_core::HRESULT,
    pub InsertScalar: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        f32,
    ) -> windows_core::HRESULT,
    pub InsertVector2: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub InsertVector3: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        windows_numerics::Vector3,
    ) -> windows_core::HRESULT,
    pub InsertVector4: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        windows_numerics::Vector4,
    ) -> windows_core::HRESULT,
    pub TryGetColor: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut Color,
        *mut CompositionGetValueStatus,
    ) -> windows_core::HRESULT,
    pub TryGetMatrix3x2: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut windows_numerics::Matrix3x2,
        *mut CompositionGetValueStatus,
    ) -> windows_core::HRESULT,
    pub TryGetMatrix4x4: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut windows_numerics::Matrix4x4,
        *mut CompositionGetValueStatus,
    ) -> windows_core::HRESULT,
    pub TryGetQuaternion: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut Quaternion,
        *mut CompositionGetValueStatus,
    ) -> windows_core::HRESULT,
    pub TryGetScalar: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut f32,
        *mut CompositionGetValueStatus,
    ) -> windows_core::HRESULT,
    pub TryGetVector2: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector2,
        *mut CompositionGetValueStatus,
    ) -> windows_core::HRESULT,
    pub TryGetVector3: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector3,
        *mut CompositionGetValueStatus,
    ) -> windows_core::HRESULT,
    pub TryGetVector4: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector4,
        *mut CompositionGetValueStatus,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionRoundedRectangleGeometry,
    ICompositionRoundedRectangleGeometry_Vtbl,
    0x8770c822_1d50_4b8b_b013_7c9a0e46935f
);
impl windows_core::RuntimeType for ICompositionRoundedRectangleGeometry {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionRoundedRectangleGeometry",
    );
}
#[repr(C)]
pub struct ICompositionRoundedRectangleGeometry_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CornerRadius: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub SetCornerRadius: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub Offset: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub SetOffset: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub Size: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub SetSize: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionScopedBatch,
    ICompositionScopedBatch_Vtbl,
    0x0d00dad0_fb07_46fd_8c72_6280d1a3d1dd
);
impl windows_core::RuntimeType for ICompositionScopedBatch {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionScopedBatch",
    );
}
#[repr(C)]
pub struct ICompositionScopedBatch_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub IsActive:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub IsEnded:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub End: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Resume: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Suspend: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Completed: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut i64,
    ) -> windows_core::HRESULT,
    pub RemoveCompleted:
        unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionShadow,
    ICompositionShadow_Vtbl,
    0x329e52e2_4335_49cc_b14a_37782d10f0c4
);
impl windows_core::RuntimeType for ICompositionShadow {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositionShadow");
}
#[repr(C)]
pub struct ICompositionShadow_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionSurface,
    ICompositionSurface_Vtbl,
    0x1527540d_42c7_47a6_a408_668f79a90dfb
);
impl windows_core::RuntimeType for ICompositionSurface {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositionSurface");
}
windows_core::imp::interface_hierarchy!(
    ICompositionSurface,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeName for ICompositionSurface {
    const NAME: &'static str = "Windows.UI.Composition.ICompositionSurface";
}
pub trait ICompositionSurface_Impl: windows_core::IUnknownImpl {}
impl ICompositionSurface_Vtbl {
    pub const fn new<Identity: ICompositionSurface_Impl, const OFFSET: isize>() -> Self {
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<Identity, ICompositionSurface, OFFSET>(),
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<ICompositionSurface as windows_core::Interface>::IID
    }
}
#[repr(C)]
pub struct ICompositionSurface_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionSurfaceBrush,
    ICompositionSurfaceBrush_Vtbl,
    0xad016d79_1e4c_4c0d_9c29_83338c87c162
);
impl windows_core::RuntimeType for ICompositionSurfaceBrush {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionSurfaceBrush",
    );
}
#[repr(C)]
pub struct ICompositionSurfaceBrush_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionTarget,
    ICompositionTarget_Vtbl,
    0xa1bea8ba_d726_4663_8129_6b5e7927ffa6
);
impl windows_core::RuntimeType for ICompositionTarget {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositionTarget");
}
#[repr(C)]
pub struct ICompositionTarget_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionViewBox,
    ICompositionViewBox_Vtbl,
    0xb440bf07_068f_4537_84c6_4ecbe019e1f4
);
impl windows_core::RuntimeType for ICompositionViewBox {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositionViewBox");
}
#[repr(C)]
pub struct ICompositionViewBox_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositor,
    ICompositor_Vtbl,
    0xb403ca50_7f8c_4e83_985f_cc45060036d8
);
impl windows_core::RuntimeType for ICompositor {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositor");
}
#[repr(C)]
pub struct ICompositor_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateColorKeyFrameAnimation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateColorBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateColorBrushWithColor: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        Color,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateContainerVisual: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateCubicBezierEasingFunction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
        windows_numerics::Vector2,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateEffectFactory: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateEffectFactoryWithProperties: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateExpressionAnimation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateExpressionAnimationWithExpression: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    )
        -> windows_core::HRESULT,
    pub CreateInsetClip: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateInsetClipWithInsets: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        f32,
        f32,
        f32,
        f32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateLinearEasingFunction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreatePropertySet: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateQuaternionKeyFrameAnimation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateScalarKeyFrameAnimation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateScopedBatch: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        CompositionBatchTypes,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateSpriteVisual: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateSurfaceBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateSurfaceBrushWithSurface: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateTargetForCurrentView: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateVector2KeyFrameAnimation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateVector3KeyFrameAnimation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateVector4KeyFrameAnimation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetCommitBatch: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        CompositionBatchTypes,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositor2,
    ICompositor2_Vtbl,
    0x735081dc_5e24_45da_a38f_e32cc349a9a0
);
impl windows_core::RuntimeType for ICompositor2 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositor2");
}
#[repr(C)]
pub struct ICompositor2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateAmbientLight: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateAnimationGroup: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBackdropBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateDistantLight: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateDropShadow: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateImplicitAnimationCollection: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateLayerVisual: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateMaskBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateNineGridBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreatePointLight: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateSpotLight: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateStepEasingFunction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateStepEasingFunctionWithStepCount: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        i32,
        *mut *mut core::ffi::c_void,
    )
        -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositor4,
    ICompositor4_Vtbl,
    0xae47e78a_7910_4425_a482_a05b758adce9
);
impl windows_core::RuntimeType for ICompositor4 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositor4");
}
#[repr(C)]
pub struct ICompositor4_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateColorGradientStop: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateColorGradientStopWithOffsetAndColor:
        unsafe extern "system" fn(
            *mut core::ffi::c_void,
            f32,
            Color,
            *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT,
    pub CreateLinearGradientBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateSpringScalarAnimation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateSpringVector2Animation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateSpringVector3Animation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositor5,
    ICompositor5_Vtbl,
    0x48ea31ad_7fcd_4076_a79c_90cc4b852c9b
);
impl windows_core::RuntimeType for ICompositor5 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositor5");
}
#[repr(C)]
pub struct ICompositor5_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    Comment: usize,
    SetComment: usize,
    GlobalPlaybackRate: usize,
    SetGlobalPlaybackRate: usize,
    CreateBounceScalarAnimation: usize,
    CreateBounceVector2Animation: usize,
    CreateBounceVector3Animation: usize,
    CreateContainerShape: usize,
    CreateEllipseGeometry: usize,
    CreateLineGeometry: usize,
    CreatePathGeometry: usize,
    CreatePathGeometryWithPath: usize,
    CreatePathKeyFrameAnimation: usize,
    CreateRectangleGeometry: usize,
    pub CreateRoundedRectangleGeometry: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositor6,
    ICompositor6_Vtbl,
    0x7a38b2bd_cec8_4eeb_830f_d8d07aedebc3
);
impl windows_core::RuntimeType for ICompositor6 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositor6");
}
#[repr(C)]
pub struct ICompositor6_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    CreateGeometricClip: usize,
    pub CreateGeometricClipWithGeometry: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositorStatics,
    ICompositorStatics_Vtbl,
    0x080db93e_121e_4d97_8b74_1dfcf91987ea
);
impl windows_core::RuntimeType for ICompositorStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositorStatics");
}
#[repr(C)]
pub struct ICompositorStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub MaxGlobalPlaybackRate:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32) -> windows_core::HRESULT,
    pub MinGlobalPlaybackRate:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IContainerVisual,
    IContainerVisual_Vtbl,
    0x02f6bc74_ed20_4773_afe6_d49b4a93db32
);
impl windows_core::RuntimeType for IContainerVisual {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IContainerVisual");
}
#[repr(C)]
pub struct IContainerVisual_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Children: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICoreDispatcher,
    ICoreDispatcher_Vtbl,
    0x60db2fa8_b705_4fde_a7d6_ebbb1891d39e
);
impl windows_core::RuntimeType for ICoreDispatcher {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Core.ICoreDispatcher");
}
#[repr(C)]
pub struct ICoreDispatcher_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICubicBezierEasingFunction,
    ICubicBezierEasingFunction_Vtbl,
    0x32350666_c1e8_44f9_96b8_c98acf0ae698
);
impl windows_core::RuntimeType for ICubicBezierEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICubicBezierEasingFunction",
    );
}
#[repr(C)]
pub struct ICubicBezierEasingFunction_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IDistantLight,
    IDistantLight_Vtbl,
    0x318cfafc_5ce3_4b55_ab5d_07a00353ac99
);
impl windows_core::RuntimeType for IDistantLight {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IDistantLight");
}
#[repr(C)]
pub struct IDistantLight_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IDropShadow,
    IDropShadow_Vtbl,
    0xcb977c07_a154_4851_85e7_a8924c84fad8
);
impl windows_core::RuntimeType for IDropShadow {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IDropShadow");
}
#[repr(C)]
pub struct IDropShadow_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IElasticEasingFunction,
    IElasticEasingFunction_Vtbl,
    0x66de6285_054e_5594_8475_c22cb51f1bd5
);
impl windows_core::RuntimeType for IElasticEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.IElasticEasingFunction",
    );
}
#[repr(C)]
pub struct IElasticEasingFunction_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IExponentialEasingFunction,
    IExponentialEasingFunction_Vtbl,
    0x6f7d1a51_98d2_5638_a34a_00486554c750
);
impl windows_core::RuntimeType for IExponentialEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.IExponentialEasingFunction",
    );
}
#[repr(C)]
pub struct IExponentialEasingFunction_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IExpressionAnimation,
    IExpressionAnimation_Vtbl,
    0x6acc5431_7d3d_4bf3_abb6_f44bdc4888c1
);
impl windows_core::RuntimeType for IExpressionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IExpressionAnimation");
}
#[repr(C)]
pub struct IExpressionAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Expression: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetExpression: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IGraphicsEffect,
    IGraphicsEffect_Vtbl,
    0xcb51c0ce_8fe6_4636_b202_861faa07d8f3
);
impl windows_core::RuntimeType for IGraphicsEffect {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Graphics.Effects.IGraphicsEffect");
}
windows_core::imp::interface_hierarchy!(
    IGraphicsEffect,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeName for IGraphicsEffect {
    const NAME: &'static str = "Windows.Graphics.Effects.IGraphicsEffect";
}
#[repr(C)]
pub struct IGraphicsEffect_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IImplicitAnimationCollection,
    IImplicitAnimationCollection_Vtbl,
    0x0598a3ff_0a92_4c9d_a427_b25519250dbf
);
impl windows_core::RuntimeType for IImplicitAnimationCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.IImplicitAnimationCollection",
    );
}
#[repr(C)]
pub struct IImplicitAnimationCollection_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IInsetClip,
    IInsetClip_Vtbl,
    0x1e73e647_84c7_477a_b474_5880e0442e15
);
impl windows_core::RuntimeType for IInsetClip {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IInsetClip");
}
#[repr(C)]
pub struct IInsetClip_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub BottomInset:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32) -> windows_core::HRESULT,
    pub SetBottomInset:
        unsafe extern "system" fn(*mut core::ffi::c_void, f32) -> windows_core::HRESULT,
    pub LeftInset:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32) -> windows_core::HRESULT,
    pub SetLeftInset:
        unsafe extern "system" fn(*mut core::ffi::c_void, f32) -> windows_core::HRESULT,
    pub RightInset:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32) -> windows_core::HRESULT,
    pub SetRightInset:
        unsafe extern "system" fn(*mut core::ffi::c_void, f32) -> windows_core::HRESULT,
    pub TopInset:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32) -> windows_core::HRESULT,
    pub SetTopInset:
        unsafe extern "system" fn(*mut core::ffi::c_void, f32) -> windows_core::HRESULT,
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IIterable<T>(windows_core::IUnknown, core::marker::PhantomData<T>)
where
    T: windows_core::RuntimeType + 'static;
impl<T: windows_core::RuntimeType + 'static> windows_core::imp::CanInto<windows_core::IUnknown>
    for IIterable<T>
{
}
impl<T: windows_core::RuntimeType + 'static> windows_core::imp::CanInto<windows_core::IInspectable>
    for IIterable<T>
{
}
unsafe impl<T: windows_core::RuntimeType + 'static> windows_core::Interface for IIterable<T> {
    type Vtable = IIterable_Vtbl<T>;
    const IID: windows_core::GUID =
        windows_core::GUID::from_signature(<Self as windows_core::RuntimeType>::SIGNATURE);
}
impl<T: windows_core::RuntimeType + 'static> windows_core::RuntimeType for IIterable<T> {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::new()
        .push_slice(b"pinterface({faa585ea-6214-4217-afda-7f46de5869b3}")
        .push_slice(b";")
        .push_other(T::SIGNATURE)
        .push_slice(b")");
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::new()
        .push_slice(b"Windows.Foundation.Collections.IIterable`1<")
        .push_other(T::NAME)
        .push_slice(b">");
}
impl<T: windows_core::RuntimeType + 'static> IIterable<T> {
    pub(crate) fn First(&self) -> windows_core::Result<windows_collections::IIterator<T>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).First)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
impl<T: windows_core::RuntimeType + 'static> windows_core::RuntimeName for IIterable<T> {
    const NAME: &'static str = "Windows.Foundation.Collections.IIterable";
    const RUNTIME_CLASS_NAME: windows_core::imp::ConstBuffer =
        <Self as windows_core::RuntimeType>::NAME;
}
pub trait IIterable_Impl<T>: windows_core::IUnknownImpl
where
    T: windows_core::RuntimeType + 'static,
{
    fn First(&self) -> windows_core::Result<windows_collections::IIterator<T>>;
}
impl<T: windows_core::RuntimeType + 'static> IIterable_Vtbl<T> {
    pub const fn new<Identity: IIterable_Impl<T>, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn First<
            T: windows_core::RuntimeType + 'static,
            Identity: IIterable_Impl<T>,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            result__: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IIterable_Impl::First(this) {
                    Ok(ok__) => {
                        result__.write(core::mem::transmute_copy(&ok__));
                        core::mem::forget(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<Identity, IIterable<T>, OFFSET>(),
            First: First::<T, Identity, OFFSET>,
            T: core::marker::PhantomData::<T>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IIterable<T> as windows_core::Interface>::IID
    }
}
#[repr(C)]
pub struct IIterable_Vtbl<T>
where
    T: windows_core::RuntimeType + 'static,
{
    pub base__: windows_core::IInspectable_Vtbl,
    pub First: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    T: core::marker::PhantomData<T>,
}
impl<T: windows_core::RuntimeType> IntoIterator for IIterable<T> {
    type Item = T;
    type IntoIter = windows_collections::BufferedIterator<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(&self)
    }
}
impl<T: windows_core::RuntimeType> IntoIterator for &IIterable<T> {
    type Item = T;
    type IntoIter = windows_collections::BufferedIterator<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        windows_collections::BufferedIterator::new(self.First().unwrap())
    }
}
windows_core::imp::define_interface!(
    IKeyFrameAnimation,
    IKeyFrameAnimation_Vtbl,
    0x126e7f22_3ae9_4540_9a8a_deae8a4a4a84
);
impl windows_core::RuntimeType for IKeyFrameAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IKeyFrameAnimation");
}
#[repr(C)]
pub struct IKeyFrameAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub DelayTime: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_time::TimeSpan,
    ) -> windows_core::HRESULT,
    pub SetDelayTime: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_time::TimeSpan,
    ) -> windows_core::HRESULT,
    pub Duration: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_time::TimeSpan,
    ) -> windows_core::HRESULT,
    pub SetDuration: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_time::TimeSpan,
    ) -> windows_core::HRESULT,
    pub IterationBehavior: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut AnimationIterationBehavior,
    ) -> windows_core::HRESULT,
    pub SetIterationBehavior: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        AnimationIterationBehavior,
    ) -> windows_core::HRESULT,
    pub IterationCount:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub SetIterationCount:
        unsafe extern "system" fn(*mut core::ffi::c_void, i32) -> windows_core::HRESULT,
    pub KeyFrameCount:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub StopBehavior: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut AnimationStopBehavior,
    ) -> windows_core::HRESULT,
    pub SetStopBehavior: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        AnimationStopBehavior,
    ) -> windows_core::HRESULT,
    pub InsertExpressionKeyFrame: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        f32,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub InsertExpressionKeyFrameWithEasingFunction:
        unsafe extern "system" fn(
            *mut core::ffi::c_void,
            f32,
            *mut core::ffi::c_void,
            *mut core::ffi::c_void,
        ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IKeyFrameAnimation2,
    IKeyFrameAnimation2_Vtbl,
    0xf4b488bb_2940_4ec0_a41a_eb6d801a2f18
);
impl windows_core::RuntimeType for IKeyFrameAnimation2 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IKeyFrameAnimation2");
}
#[repr(C)]
pub struct IKeyFrameAnimation2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Direction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut AnimationDirection,
    ) -> windows_core::HRESULT,
    pub SetDirection: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        AnimationDirection,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IKeyFrameAnimation3,
    IKeyFrameAnimation3_Vtbl,
    0x845bf0b4_d8de_462f_8753_c80d43c6ff5a
);
impl windows_core::RuntimeType for IKeyFrameAnimation3 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IKeyFrameAnimation3");
}
#[repr(C)]
pub struct IKeyFrameAnimation3_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub DelayBehavior: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut AnimationDelayBehavior,
    ) -> windows_core::HRESULT,
    pub SetDelayBehavior: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        AnimationDelayBehavior,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ILayerVisual,
    ILayerVisual_Vtbl,
    0xaf843985_0444_4887_8e83_b40b253f822c
);
impl windows_core::RuntimeType for ILayerVisual {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ILayerVisual");
}
#[repr(C)]
pub struct ILayerVisual_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ILinearEasingFunction,
    ILinearEasingFunction_Vtbl,
    0x9400975a_c7a6_46b3_acf7_1a268a0a117d
);
impl windows_core::RuntimeType for ILinearEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ILinearEasingFunction");
}
#[repr(C)]
pub struct ILinearEasingFunction_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IMap<K, V>(
    windows_core::IUnknown,
    core::marker::PhantomData<K>,
    core::marker::PhantomData<V>,
)
where
    K: windows_core::RuntimeType + 'static,
    V: windows_core::RuntimeType + 'static;
impl<K: windows_core::RuntimeType + 'static, V: windows_core::RuntimeType + 'static>
    windows_core::imp::CanInto<windows_core::IUnknown> for IMap<K, V>
{
}
impl<K: windows_core::RuntimeType + 'static, V: windows_core::RuntimeType + 'static>
    windows_core::imp::CanInto<windows_core::IInspectable> for IMap<K, V>
{
}
unsafe impl<K: windows_core::RuntimeType + 'static, V: windows_core::RuntimeType + 'static>
    windows_core::Interface for IMap<K, V>
{
    type Vtable = IMap_Vtbl<K, V>;
    const IID: windows_core::GUID =
        windows_core::GUID::from_signature(<Self as windows_core::RuntimeType>::SIGNATURE);
}
impl<K: windows_core::RuntimeType + 'static, V: windows_core::RuntimeType + 'static>
    windows_core::RuntimeType for IMap<K, V>
{
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::new()
        .push_slice(b"pinterface({3c2925fe-8519-45c1-aa79-197b6718c1c1}")
        .push_slice(b";")
        .push_other(K::SIGNATURE)
        .push_slice(b";")
        .push_other(V::SIGNATURE)
        .push_slice(b")");
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::new()
        .push_slice(b"Windows.Foundation.Collections.IMap`2<")
        .push_other(K::NAME)
        .push_slice(b", ")
        .push_other(V::NAME)
        .push_slice(b">");
}
impl<K: windows_core::RuntimeType + 'static, V: windows_core::RuntimeType + 'static>
    windows_core::imp::CanInto<IIterable<windows_collections::IKeyValuePair<K, V>>> for IMap<K, V>
{
    const QUERY: bool = true;
}
impl<K: windows_core::RuntimeType + 'static, V: windows_core::RuntimeType + 'static> IMap<K, V> {
    pub(crate) fn Lookup<P0>(&self, key: P0) -> windows_core::Result<V>
    where
        P0: windows_core::Param<K>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Lookup)(
                windows_core::Interface::as_raw(self),
                key.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Size(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Size)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn HasKey<P0>(&self, key: P0) -> windows_core::Result<bool>
    where
        P0: windows_core::Param<K>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).HasKey)(
                windows_core::Interface::as_raw(self),
                key.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn GetView(&self) -> windows_core::Result<windows_collections::IMapView<K, V>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetView)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Insert<P0, P1>(&self, key: P0, value: P1) -> windows_core::Result<bool>
    where
        P0: windows_core::Param<K>,
        P1: windows_core::Param<V>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Insert)(
                windows_core::Interface::as_raw(self),
                key.param().abi(),
                value.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn Remove<P0>(&self, key: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<K>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Remove)(
                windows_core::Interface::as_raw(self),
                key.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn Clear(&self) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).Clear)(windows_core::Interface::as_raw(self))
                .ok()
        }
    }
    pub(crate) fn First(
        &self,
    ) -> windows_core::Result<
        windows_collections::IIterator<windows_collections::IKeyValuePair<K, V>>,
    > {
        let this = &windows_core::Interface::cast::<
            IIterable<windows_collections::IKeyValuePair<K, V>>,
        >(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).First)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
impl<K: windows_core::RuntimeType + 'static, V: windows_core::RuntimeType + 'static> IntoIterator
    for IMap<K, V>
{
    type Item = windows_collections::IKeyValuePair<K, V>;
    type IntoIter = windows_collections::BufferedIterator<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(&self)
    }
}
impl<K: windows_core::RuntimeType + 'static, V: windows_core::RuntimeType + 'static> IntoIterator
    for &IMap<K, V>
{
    type Item = windows_collections::IKeyValuePair<K, V>;
    type IntoIter = windows_collections::BufferedIterator<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        windows_collections::BufferedIterator::new(self.First().unwrap())
    }
}
impl<K: windows_core::RuntimeType + 'static, V: windows_core::RuntimeType + 'static>
    windows_core::RuntimeName for IMap<K, V>
{
    const NAME: &'static str = "Windows.Foundation.Collections.IMap";
    const RUNTIME_CLASS_NAME: windows_core::imp::ConstBuffer =
        <Self as windows_core::RuntimeType>::NAME;
}
pub trait IMap_Impl<K, V>: IIterable_Impl<windows_collections::IKeyValuePair<K, V>>
where
    K: windows_core::RuntimeType + 'static,
    V: windows_core::RuntimeType + 'static,
{
    fn Lookup(&self, key: windows_core::Ref<K>) -> windows_core::Result<V>;
    fn Size(&self) -> windows_core::Result<u32>;
    fn HasKey(&self, key: windows_core::Ref<K>) -> windows_core::Result<bool>;
    fn GetView(&self) -> windows_core::Result<windows_collections::IMapView<K, V>>;
    fn Insert(
        &self,
        key: windows_core::Ref<K>,
        value: windows_core::Ref<V>,
    ) -> windows_core::Result<bool>;
    fn Remove(&self, key: windows_core::Ref<K>) -> windows_core::Result<()>;
    fn Clear(&self) -> windows_core::Result<()>;
}
impl<K: windows_core::RuntimeType + 'static, V: windows_core::RuntimeType + 'static>
    IMap_Vtbl<K, V>
{
    pub const fn new<Identity: IMap_Impl<K, V>, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn Lookup<
            K: windows_core::RuntimeType + 'static,
            V: windows_core::RuntimeType + 'static,
            Identity: IMap_Impl<K, V>,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            key: windows_core::imp::AbiType<K>,
            result__: *mut windows_core::imp::AbiType<V>,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IMap_Impl::Lookup(this, core::mem::transmute_copy(&key)) {
                    Ok(ok__) => {
                        result__.write(core::mem::transmute_copy(&ok__));
                        core::mem::forget(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn Size<
            K: windows_core::RuntimeType + 'static,
            V: windows_core::RuntimeType + 'static,
            Identity: IMap_Impl<K, V>,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            result__: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IMap_Impl::Size(this) {
                    Ok(ok__) => {
                        result__.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn HasKey<
            K: windows_core::RuntimeType + 'static,
            V: windows_core::RuntimeType + 'static,
            Identity: IMap_Impl<K, V>,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            key: windows_core::imp::AbiType<K>,
            result__: *mut bool,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IMap_Impl::HasKey(this, core::mem::transmute_copy(&key)) {
                    Ok(ok__) => {
                        result__.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetView<
            K: windows_core::RuntimeType + 'static,
            V: windows_core::RuntimeType + 'static,
            Identity: IMap_Impl<K, V>,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            result__: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IMap_Impl::GetView(this) {
                    Ok(ok__) => {
                        result__.write(core::mem::transmute_copy(&ok__));
                        core::mem::forget(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn Insert<
            K: windows_core::RuntimeType + 'static,
            V: windows_core::RuntimeType + 'static,
            Identity: IMap_Impl<K, V>,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            key: windows_core::imp::AbiType<K>,
            value: windows_core::imp::AbiType<V>,
            result__: *mut bool,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IMap_Impl::Insert(
                    this,
                    core::mem::transmute_copy(&key),
                    core::mem::transmute_copy(&value),
                ) {
                    Ok(ok__) => {
                        result__.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn Remove<
            K: windows_core::RuntimeType + 'static,
            V: windows_core::RuntimeType + 'static,
            Identity: IMap_Impl<K, V>,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            key: windows_core::imp::AbiType<K>,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IMap_Impl::Remove(this, core::mem::transmute_copy(&key)).into()
            }
        }
        unsafe extern "system" fn Clear<
            K: windows_core::RuntimeType + 'static,
            V: windows_core::RuntimeType + 'static,
            Identity: IMap_Impl<K, V>,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IMap_Impl::Clear(this).into()
            }
        }
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<Identity, IMap<K, V>, OFFSET>(),
            Lookup: Lookup::<K, V, Identity, OFFSET>,
            Size: Size::<K, V, Identity, OFFSET>,
            HasKey: HasKey::<K, V, Identity, OFFSET>,
            GetView: GetView::<K, V, Identity, OFFSET>,
            Insert: Insert::<K, V, Identity, OFFSET>,
            Remove: Remove::<K, V, Identity, OFFSET>,
            Clear: Clear::<K, V, Identity, OFFSET>,
            K: core::marker::PhantomData::<K>,
            V: core::marker::PhantomData::<V>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IMap<K, V> as windows_core::Interface>::IID
    }
}
#[repr(C)]
pub struct IMap_Vtbl<K, V>
where
    K: windows_core::RuntimeType + 'static,
    V: windows_core::RuntimeType + 'static,
{
    pub base__: windows_core::IInspectable_Vtbl,
    pub Lookup: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::imp::AbiType<K>,
        *mut windows_core::imp::AbiType<V>,
    ) -> windows_core::HRESULT,
    pub Size: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
    pub HasKey: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::imp::AbiType<K>,
        *mut bool,
    ) -> windows_core::HRESULT,
    pub GetView: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub Insert: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::imp::AbiType<K>,
        windows_core::imp::AbiType<V>,
        *mut bool,
    ) -> windows_core::HRESULT,
    pub Remove: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::imp::AbiType<K>,
    ) -> windows_core::HRESULT,
    pub Clear: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    K: core::marker::PhantomData<K>,
    V: core::marker::PhantomData<V>,
}
windows_core::imp::define_interface!(
    INaturalMotionAnimation,
    INaturalMotionAnimation_Vtbl,
    0x438de12d_769b_4821_a949_284a6547e873
);
impl windows_core::RuntimeType for INaturalMotionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.INaturalMotionAnimation",
    );
}
#[repr(C)]
pub struct INaturalMotionAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IPointLight,
    IPointLight_Vtbl,
    0xb18545b3_0c5a_4ab0_bedc_4f3546948272
);
impl windows_core::RuntimeType for IPointLight {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IPointLight");
}
#[repr(C)]
pub struct IPointLight_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IPowerEasingFunction,
    IPowerEasingFunction_Vtbl,
    0xc3ff53d6_138b_5815_891a_b7f615ccc563
);
impl windows_core::RuntimeType for IPowerEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IPowerEasingFunction");
}
#[repr(C)]
pub struct IPowerEasingFunction_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IQuaternionKeyFrameAnimation,
    IQuaternionKeyFrameAnimation_Vtbl,
    0x404e5835_ecf6_4240_8520_671279cf36bc
);
impl windows_core::RuntimeType for IQuaternionKeyFrameAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.IQuaternionKeyFrameAnimation",
    );
}
#[repr(C)]
pub struct IQuaternionKeyFrameAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IScalarKeyFrameAnimation,
    IScalarKeyFrameAnimation_Vtbl,
    0xae288fa9_252c_4b95_a725_bf85e38000a1
);
impl windows_core::RuntimeType for IScalarKeyFrameAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.IScalarKeyFrameAnimation",
    );
}
#[repr(C)]
pub struct IScalarKeyFrameAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub InsertKeyFrame:
        unsafe extern "system" fn(*mut core::ffi::c_void, f32, f32) -> windows_core::HRESULT,
    pub InsertKeyFrameWithEasingFunction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        f32,
        f32,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IScalarNaturalMotionAnimation,
    IScalarNaturalMotionAnimation_Vtbl,
    0x94a94581_bf92_495b_b5bd_d2c659430737
);
impl windows_core::RuntimeType for IScalarNaturalMotionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.IScalarNaturalMotionAnimation",
    );
}
#[repr(C)]
pub struct IScalarNaturalMotionAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ISineEasingFunction,
    ISineEasingFunction_Vtbl,
    0xf1b518bf_9563_5474_bd13_44b2df4b1d58
);
impl windows_core::RuntimeType for ISineEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ISineEasingFunction");
}
#[repr(C)]
pub struct ISineEasingFunction_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ISpotLight,
    ISpotLight_Vtbl,
    0x5a9fe273_44a1_4f95_a422_8fa5116bdb44
);
impl windows_core::RuntimeType for ISpotLight {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ISpotLight");
}
#[repr(C)]
pub struct ISpotLight_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ISpringScalarNaturalMotionAnimation,
    ISpringScalarNaturalMotionAnimation_Vtbl,
    0x0572a95f_37f9_4fbe_b87b_5cd03a89501c
);
impl windows_core::RuntimeType for ISpringScalarNaturalMotionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ISpringScalarNaturalMotionAnimation",
    );
}
#[repr(C)]
pub struct ISpringScalarNaturalMotionAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ISpringVector2NaturalMotionAnimation,
    ISpringVector2NaturalMotionAnimation_Vtbl,
    0x23f494b5_ee73_4f0f_a423_402b946df4b3
);
impl windows_core::RuntimeType for ISpringVector2NaturalMotionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ISpringVector2NaturalMotionAnimation",
    );
}
#[repr(C)]
pub struct ISpringVector2NaturalMotionAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ISpringVector3NaturalMotionAnimation,
    ISpringVector3NaturalMotionAnimation_Vtbl,
    0x6c8749df_d57b_4794_8e2d_cecb11e194e5
);
impl windows_core::RuntimeType for ISpringVector3NaturalMotionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ISpringVector3NaturalMotionAnimation",
    );
}
#[repr(C)]
pub struct ISpringVector3NaturalMotionAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ISpriteVisual,
    ISpriteVisual_Vtbl,
    0x08e05581_1ad1_4f97_9757_402d76e4233b
);
impl windows_core::RuntimeType for ISpriteVisual {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ISpriteVisual");
}
#[repr(C)]
pub struct ISpriteVisual_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Brush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IStepEasingFunction,
    IStepEasingFunction_Vtbl,
    0xd0caa74b_560c_4a0b_a5f6_206ca8c3ecd6
);
impl windows_core::RuntimeType for IStepEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IStepEasingFunction");
}
#[repr(C)]
pub struct IStepEasingFunction_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IVector2KeyFrameAnimation,
    IVector2KeyFrameAnimation_Vtbl,
    0xdf414515_4e29_4f11_b55e_bf2a6eb36294
);
impl windows_core::RuntimeType for IVector2KeyFrameAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.IVector2KeyFrameAnimation",
    );
}
#[repr(C)]
pub struct IVector2KeyFrameAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub InsertKeyFrame: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        f32,
        windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub InsertKeyFrameWithEasingFunction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        f32,
        windows_numerics::Vector2,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IVector2NaturalMotionAnimation,
    IVector2NaturalMotionAnimation_Vtbl,
    0x0f3e0b7d_e512_479d_a00c_77c93a30a395
);
impl windows_core::RuntimeType for IVector2NaturalMotionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.IVector2NaturalMotionAnimation",
    );
}
#[repr(C)]
pub struct IVector2NaturalMotionAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IVector3KeyFrameAnimation,
    IVector3KeyFrameAnimation_Vtbl,
    0xc8039daa_a281_43c2_a73d_b68e3c533c40
);
impl windows_core::RuntimeType for IVector3KeyFrameAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.IVector3KeyFrameAnimation",
    );
}
#[repr(C)]
pub struct IVector3KeyFrameAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub InsertKeyFrame: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        f32,
        windows_numerics::Vector3,
    ) -> windows_core::HRESULT,
    pub InsertKeyFrameWithEasingFunction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        f32,
        windows_numerics::Vector3,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IVector3NaturalMotionAnimation,
    IVector3NaturalMotionAnimation_Vtbl,
    0x9c17042c_e2ca_45ad_969e_4e78b7b9ad41
);
impl windows_core::RuntimeType for IVector3NaturalMotionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.IVector3NaturalMotionAnimation",
    );
}
#[repr(C)]
pub struct IVector3NaturalMotionAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IVector4KeyFrameAnimation,
    IVector4KeyFrameAnimation_Vtbl,
    0x2457945b_addd_4385_9606_b6a3d5e4e1b9
);
impl windows_core::RuntimeType for IVector4KeyFrameAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.IVector4KeyFrameAnimation",
    );
}
#[repr(C)]
pub struct IVector4KeyFrameAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IVisual,
    IVisual_Vtbl,
    0x117e202d_a859_4c89_873b_c2aa566788e3
);
impl windows_core::RuntimeType for IVisual {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IVisual");
}
#[repr(C)]
pub struct IVisual_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub AnchorPoint: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub SetAnchorPoint: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub BackfaceVisibility: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut CompositionBackfaceVisibility,
    ) -> windows_core::HRESULT,
    pub SetBackfaceVisibility: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        CompositionBackfaceVisibility,
    ) -> windows_core::HRESULT,
    pub BorderMode: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut CompositionBorderMode,
    ) -> windows_core::HRESULT,
    pub SetBorderMode: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        CompositionBorderMode,
    ) -> windows_core::HRESULT,
    pub CenterPoint: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector3,
    ) -> windows_core::HRESULT,
    pub SetCenterPoint: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector3,
    ) -> windows_core::HRESULT,
    pub Clip: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetClip: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CompositeMode: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut CompositionCompositeMode,
    ) -> windows_core::HRESULT,
    pub SetCompositeMode: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        CompositionCompositeMode,
    ) -> windows_core::HRESULT,
    pub IsVisible:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsVisible:
        unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
    pub Offset: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector3,
    ) -> windows_core::HRESULT,
    pub SetOffset: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector3,
    ) -> windows_core::HRESULT,
    pub Opacity:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32) -> windows_core::HRESULT,
    pub SetOpacity: unsafe extern "system" fn(*mut core::ffi::c_void, f32) -> windows_core::HRESULT,
    pub Orientation:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut Quaternion) -> windows_core::HRESULT,
    pub SetOrientation:
        unsafe extern "system" fn(*mut core::ffi::c_void, Quaternion) -> windows_core::HRESULT,
    pub Parent: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RotationAngle:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32) -> windows_core::HRESULT,
    pub SetRotationAngle:
        unsafe extern "system" fn(*mut core::ffi::c_void, f32) -> windows_core::HRESULT,
    pub RotationAngleInDegrees:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32) -> windows_core::HRESULT,
    pub SetRotationAngleInDegrees:
        unsafe extern "system" fn(*mut core::ffi::c_void, f32) -> windows_core::HRESULT,
    pub RotationAxis: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector3,
    ) -> windows_core::HRESULT,
    pub SetRotationAxis: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector3,
    ) -> windows_core::HRESULT,
    pub Scale: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector3,
    ) -> windows_core::HRESULT,
    pub SetScale: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector3,
    ) -> windows_core::HRESULT,
    pub Size: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub SetSize: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub TransformMatrix: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Matrix4x4,
    ) -> windows_core::HRESULT,
    pub SetTransformMatrix: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Matrix4x4,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IVisual2,
    IVisual2_Vtbl,
    0x3052b611_56c3_4c3e_8bf3_f6e1ad473f06
);
impl windows_core::RuntimeType for IVisual2 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IVisual2");
}
#[repr(C)]
pub struct IVisual2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub ParentForTransform: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetParentForTransform: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RelativeOffsetAdjustment: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector3,
    ) -> windows_core::HRESULT,
    pub SetRelativeOffsetAdjustment: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector3,
    ) -> windows_core::HRESULT,
    pub RelativeSizeAdjustment: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    pub SetRelativeSizeAdjustment: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IVisualCollection,
    IVisualCollection_Vtbl,
    0x8b745505_fd3e_4a98_84a8_e949468c6bcb
);
impl windows_core::RuntimeType for IVisualCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IVisualCollection");
}
#[repr(C)]
pub struct IVisualCollection_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Count: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub InsertAbove: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub InsertAtBottom: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub InsertAtTop: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub InsertBelow: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub Remove: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RemoveAll: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImplicitAnimationCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ImplicitAnimationCollection,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(ImplicitAnimationCollection, IAnimationObject, IIterable < windows_collections::IKeyValuePair < windows_core::HSTRING, ICompositionAnimationBase > >, IMap < windows_core::HSTRING, ICompositionAnimationBase >, CompositionObject);
impl ImplicitAnimationCollection {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<Self> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Self>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn First(
        &self,
    ) -> windows_core::Result<
        windows_collections::IIterator<
            windows_collections::IKeyValuePair<windows_core::HSTRING, ICompositionAnimationBase>,
        >,
    > {
        let this = &windows_core::Interface::cast::<
            IIterable<
                windows_collections::IKeyValuePair<
                    windows_core::HSTRING,
                    ICompositionAnimationBase,
                >,
            >,
        >(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).First)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Lookup(
        &self,
        key: &windows_core::HSTRING,
    ) -> windows_core::Result<ICompositionAnimationBase> {
        let this = &windows_core::Interface::cast::<
            IMap<windows_core::HSTRING, ICompositionAnimationBase>,
        >(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Lookup)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Size(&self) -> windows_core::Result<u32> {
        let this = &windows_core::Interface::cast::<
            IMap<windows_core::HSTRING, ICompositionAnimationBase>,
        >(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Size)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn HasKey(&self, key: &windows_core::HSTRING) -> windows_core::Result<bool> {
        let this = &windows_core::Interface::cast::<
            IMap<windows_core::HSTRING, ICompositionAnimationBase>,
        >(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).HasKey)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn GetView(
        &self,
    ) -> windows_core::Result<
        windows_collections::IMapView<windows_core::HSTRING, ICompositionAnimationBase>,
    > {
        let this = &windows_core::Interface::cast::<
            IMap<windows_core::HSTRING, ICompositionAnimationBase>,
        >(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetView)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Insert<P1>(
        &self,
        key: &windows_core::HSTRING,
        value: P1,
    ) -> windows_core::Result<bool>
    where
        P1: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<
            IMap<windows_core::HSTRING, ICompositionAnimationBase>,
        >(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Insert)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn Remove(&self, key: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<
            IMap<windows_core::HSTRING, ICompositionAnimationBase>,
        >(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).Remove)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
            )
            .ok()
        }
    }
    pub(crate) fn Clear(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<
            IMap<windows_core::HSTRING, ICompositionAnimationBase>,
        >(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).Clear)(windows_core::Interface::as_raw(this))
                .ok()
        }
    }
}
impl windows_core::RuntimeType for ImplicitAnimationCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IImplicitAnimationCollection>();
}
unsafe impl windows_core::Interface for ImplicitAnimationCollection {
    type Vtable = <IImplicitAnimationCollection as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IImplicitAnimationCollection as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for ImplicitAnimationCollection {
    const NAME: &'static str = "Windows.UI.Composition.ImplicitAnimationCollection";
}
unsafe impl Send for ImplicitAnimationCollection {}
unsafe impl Sync for ImplicitAnimationCollection {}
impl IntoIterator for ImplicitAnimationCollection {
    type Item =
        windows_collections::IKeyValuePair<windows_core::HSTRING, ICompositionAnimationBase>;
    type IntoIter = windows_collections::BufferedIterator<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(&self)
    }
}
impl IntoIterator for &ImplicitAnimationCollection {
    type Item =
        windows_collections::IKeyValuePair<windows_core::HSTRING, ICompositionAnimationBase>;
    type IntoIter = windows_collections::BufferedIterator<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        windows_collections::BufferedIterator::new(self.First().unwrap())
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsetClip(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    InsetClip,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    InsetClip,
    IAnimationObject,
    CompositionClip,
    CompositionObject
);
impl InsetClip {
    pub(crate) fn AnchorPoint(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).AnchorPoint)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetAnchorPoint(
        &self,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetAnchorPoint)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn CenterPoint(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CenterPoint)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetCenterPoint(
        &self,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetCenterPoint)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Offset(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Offset)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetOffset(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetOffset)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RotationAngle(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RotationAngle)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAngle(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRotationAngle)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RotationAngleInDegrees(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RotationAngleInDegrees)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAngleInDegrees(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRotationAngleInDegrees)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Scale(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Scale)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetScale(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScale)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn TransformMatrix(&self) -> windows_core::Result<windows_numerics::Matrix3x2> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).TransformMatrix)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetTransformMatrix(
        &self,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionClip2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTransformMatrix)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn BottomInset(&self) -> windows_core::Result<f32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).BottomInset)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetBottomInset(&self, value: f32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetBottomInset)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn LeftInset(&self) -> windows_core::Result<f32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).LeftInset)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetLeftInset(&self, value: f32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetLeftInset)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RightInset(&self) -> windows_core::Result<f32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RightInset)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRightInset(&self, value: f32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetRightInset)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn TopInset(&self) -> windows_core::Result<f32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TopInset)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetTopInset(&self, value: f32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetTopInset)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for InsetClip {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IInsetClip>();
}
unsafe impl windows_core::Interface for InsetClip {
    type Vtable = <IInsetClip as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IInsetClip as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for InsetClip {
    const NAME: &'static str = "Windows.UI.Composition.InsetClip";
}
unsafe impl Send for InsetClip {}
unsafe impl Sync for InsetClip {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyFrameAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    KeyFrameAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    KeyFrameAnimation,
    IAnimationObject,
    ICompositionAnimationBase,
    CompositionAnimation,
    CompositionObject
);
impl KeyFrameAnimation {
    pub(crate) fn ClearAllParameters(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearAllParameters)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub(crate) fn ClearParameter(&self, key: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
            )
            .ok()
        }
    }
    pub(crate) fn SetColorParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Color,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetColorParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix3x2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix3x2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix4x4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix4x4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetQuaternionParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Quaternion,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetQuaternionParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetReferenceParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScalarParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector3Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector3Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetBooleanParameter(
        &self,
        key: &windows_core::HSTRING,
        value: bool,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBooleanParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Target(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Target)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetTarget(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTarget)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn DelayTime(&self) -> windows_core::Result<windows_time::TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DelayTime)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDelayTime(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetDelayTime)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Duration(&self) -> windows_core::Result<windows_time::TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Duration)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDuration(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetDuration)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IterationBehavior(&self) -> windows_core::Result<AnimationIterationBehavior> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IterationBehavior)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIterationBehavior(
        &self,
        value: AnimationIterationBehavior,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIterationBehavior)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IterationCount(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IterationCount)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIterationCount(&self, value: i32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIterationCount)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn KeyFrameCount(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).KeyFrameCount)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn StopBehavior(&self) -> windows_core::Result<AnimationStopBehavior> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).StopBehavior)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetStopBehavior(&self, value: AnimationStopBehavior) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetStopBehavior)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertExpressionKeyFrame(
        &self,
        normalizedprogresskey: f32,
        value: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).InsertExpressionKeyFrame)(
                windows_core::Interface::as_raw(self),
                normalizedprogresskey,
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn InsertExpressionKeyFrameWithEasingFunction<P2>(
        &self,
        normalizedprogresskey: f32,
        value: &windows_core::HSTRING,
        easingfunction: P2,
    ) -> windows_core::Result<()>
    where
        P2: windows_core::Param<CompositionEasingFunction>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).InsertExpressionKeyFrameWithEasingFunction)(
                windows_core::Interface::as_raw(self),
                normalizedprogresskey,
                core::mem::transmute_copy(value),
                easingfunction.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn Direction(&self) -> windows_core::Result<AnimationDirection> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Direction)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDirection(&self, value: AnimationDirection) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDirection)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn DelayBehavior(&self) -> windows_core::Result<AnimationDelayBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation3>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).DelayBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDelayBehavior(
        &self,
        value: AnimationDelayBehavior,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation3>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDelayBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for KeyFrameAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IKeyFrameAnimation>();
}
unsafe impl windows_core::Interface for KeyFrameAnimation {
    type Vtable = <IKeyFrameAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IKeyFrameAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for KeyFrameAnimation {
    const NAME: &'static str = "Windows.UI.Composition.KeyFrameAnimation";
}
unsafe impl Send for KeyFrameAnimation {}
unsafe impl Sync for KeyFrameAnimation {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayerVisual(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    LayerVisual,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(LayerVisual, ContainerVisual, Visual, CompositionObject);
impl LayerVisual {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn Children(&self) -> windows_core::Result<VisualCollection> {
        let this = &windows_core::Interface::cast::<IContainerVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Children)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn AnchorPoint(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).AnchorPoint)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetAnchorPoint(
        &self,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetAnchorPoint)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn BackfaceVisibility(&self) -> windows_core::Result<CompositionBackfaceVisibility> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).BackfaceVisibility)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetBackfaceVisibility(
        &self,
        value: CompositionBackfaceVisibility,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBackfaceVisibility)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn BorderMode(&self) -> windows_core::Result<CompositionBorderMode> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).BorderMode)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetBorderMode(&self, value: CompositionBorderMode) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBorderMode)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn CenterPoint(&self) -> windows_core::Result<windows_numerics::Vector3> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CenterPoint)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetCenterPoint(
        &self,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetCenterPoint)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Clip(&self) -> windows_core::Result<CompositionClip> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Clip)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetClip<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<CompositionClip>,
    {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetClip)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn CompositeMode(&self) -> windows_core::Result<CompositionCompositeMode> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CompositeMode)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetCompositeMode(
        &self,
        value: CompositionCompositeMode,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetCompositeMode)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IsVisible(&self) -> windows_core::Result<bool> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IsVisible)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIsVisible(&self, value: bool) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIsVisible)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Offset(&self) -> windows_core::Result<windows_numerics::Vector3> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Offset)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetOffset(&self, value: windows_numerics::Vector3) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetOffset)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Opacity(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Opacity)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetOpacity(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetOpacity)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Orientation(&self) -> windows_core::Result<Quaternion> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Orientation)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetOrientation(&self, value: Quaternion) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetOrientation)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Parent(&self) -> windows_core::Result<ContainerVisual> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Parent)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn RotationAngle(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RotationAngle)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAngle(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRotationAngle)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RotationAngleInDegrees(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RotationAngleInDegrees)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAngleInDegrees(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRotationAngleInDegrees)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RotationAxis(&self) -> windows_core::Result<windows_numerics::Vector3> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RotationAxis)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAxis(
        &self,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRotationAxis)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Scale(&self) -> windows_core::Result<windows_numerics::Vector3> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Scale)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetScale(&self, value: windows_numerics::Vector3) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScale)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Size(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Size)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetSize(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetSize)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn TransformMatrix(&self) -> windows_core::Result<windows_numerics::Matrix4x4> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).TransformMatrix)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetTransformMatrix(
        &self,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTransformMatrix)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn ParentForTransform(&self) -> windows_core::Result<Visual> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ParentForTransform)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetParentForTransform<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Visual>,
    {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetParentForTransform)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn RelativeOffsetAdjustment(
        &self,
    ) -> windows_core::Result<windows_numerics::Vector3> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RelativeOffsetAdjustment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRelativeOffsetAdjustment(
        &self,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRelativeOffsetAdjustment)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RelativeSizeAdjustment(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RelativeSizeAdjustment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRelativeSizeAdjustment(
        &self,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRelativeSizeAdjustment)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for LayerVisual {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ILayerVisual>();
}
unsafe impl windows_core::Interface for LayerVisual {
    type Vtable = <ILayerVisual as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ILayerVisual as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for LayerVisual {
    const NAME: &'static str = "Windows.UI.Composition.LayerVisual";
}
unsafe impl Send for LayerVisual {}
unsafe impl Sync for LayerVisual {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinearEasingFunction(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    LinearEasingFunction,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    LinearEasingFunction,
    IAnimationObject,
    CompositionEasingFunction,
    CompositionObject
);
impl LinearEasingFunction {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for LinearEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ILinearEasingFunction>();
}
unsafe impl windows_core::Interface for LinearEasingFunction {
    type Vtable = <ILinearEasingFunction as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ILinearEasingFunction as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for LinearEasingFunction {
    const NAME: &'static str = "Windows.UI.Composition.LinearEasingFunction";
}
unsafe impl Send for LinearEasingFunction {}
unsafe impl Sync for LinearEasingFunction {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NaturalMotionAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    NaturalMotionAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    NaturalMotionAnimation,
    CompositionAnimation,
    CompositionObject
);
impl NaturalMotionAnimation {
    pub(crate) fn ClearAllParameters(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearAllParameters)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub(crate) fn ClearParameter(&self, key: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
            )
            .ok()
        }
    }
    pub(crate) fn SetColorParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Color,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetColorParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix3x2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix3x2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix4x4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix4x4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetQuaternionParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Quaternion,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetQuaternionParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetReferenceParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScalarParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector3Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector3Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetBooleanParameter(
        &self,
        key: &windows_core::HSTRING,
        value: bool,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBooleanParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Target(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Target)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetTarget(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTarget)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for NaturalMotionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, INaturalMotionAnimation>();
}
unsafe impl windows_core::Interface for NaturalMotionAnimation {
    type Vtable = <INaturalMotionAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <INaturalMotionAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for NaturalMotionAnimation {
    const NAME: &'static str = "Windows.UI.Composition.NaturalMotionAnimation";
}
unsafe impl Send for NaturalMotionAnimation {}
unsafe impl Sync for NaturalMotionAnimation {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PointLight(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    PointLight,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(PointLight, CompositionLight, CompositionObject);
impl PointLight {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for PointLight {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IPointLight>();
}
unsafe impl windows_core::Interface for PointLight {
    type Vtable = <IPointLight as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IPointLight as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for PointLight {
    const NAME: &'static str = "Windows.UI.Composition.PointLight";
}
unsafe impl Send for PointLight {}
unsafe impl Sync for PointLight {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PowerEasingFunction(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    PowerEasingFunction,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    PowerEasingFunction,
    CompositionEasingFunction,
    CompositionObject
);
impl PowerEasingFunction {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for PowerEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IPowerEasingFunction>();
}
unsafe impl windows_core::Interface for PowerEasingFunction {
    type Vtable = <IPowerEasingFunction as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IPowerEasingFunction as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for PowerEasingFunction {
    const NAME: &'static str = "Windows.UI.Composition.PowerEasingFunction";
}
unsafe impl Send for PowerEasingFunction {}
unsafe impl Sync for PowerEasingFunction {}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Quaternion {
    pub X: f32,
    pub Y: f32,
    pub Z: f32,
    pub W: f32,
}
impl windows_core::imp::TypeKind for Quaternion {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for Quaternion {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"struct(Windows.Foundation.Numerics.Quaternion;f4;f4;f4;f4)",
    );
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Foundation.Numerics.Quaternion");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuaternionKeyFrameAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    QuaternionKeyFrameAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    QuaternionKeyFrameAnimation,
    KeyFrameAnimation,
    CompositionAnimation,
    CompositionObject
);
impl QuaternionKeyFrameAnimation {
    pub(crate) fn ClearAllParameters(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearAllParameters)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub(crate) fn ClearParameter(&self, key: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
            )
            .ok()
        }
    }
    pub(crate) fn SetColorParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Color,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetColorParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix3x2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix3x2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix4x4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix4x4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetQuaternionParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Quaternion,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetQuaternionParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetReferenceParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScalarParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector3Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector3Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetBooleanParameter(
        &self,
        key: &windows_core::HSTRING,
        value: bool,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBooleanParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Target(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Target)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetTarget(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTarget)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn DelayTime(&self) -> windows_core::Result<windows_time::TimeSpan> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).DelayTime)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDelayTime(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDelayTime)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Duration(&self) -> windows_core::Result<windows_time::TimeSpan> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Duration)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDuration(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDuration)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IterationBehavior(&self) -> windows_core::Result<AnimationIterationBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IterationBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIterationBehavior(
        &self,
        value: AnimationIterationBehavior,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIterationBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IterationCount(&self) -> windows_core::Result<i32> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IterationCount)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIterationCount(&self, value: i32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIterationCount)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn KeyFrameCount(&self) -> windows_core::Result<i32> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).KeyFrameCount)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn StopBehavior(&self) -> windows_core::Result<AnimationStopBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).StopBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetStopBehavior(&self, value: AnimationStopBehavior) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetStopBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertExpressionKeyFrame(
        &self,
        normalizedprogresskey: f32,
        value: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).InsertExpressionKeyFrame)(
                windows_core::Interface::as_raw(this),
                normalizedprogresskey,
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn InsertExpressionKeyFrameWithEasingFunction<P2>(
        &self,
        normalizedprogresskey: f32,
        value: &windows_core::HSTRING,
        easingfunction: P2,
    ) -> windows_core::Result<()>
    where
        P2: windows_core::Param<CompositionEasingFunction>,
    {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).InsertExpressionKeyFrameWithEasingFunction)(
                windows_core::Interface::as_raw(this),
                normalizedprogresskey,
                core::mem::transmute_copy(value),
                easingfunction.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn Direction(&self) -> windows_core::Result<AnimationDirection> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Direction)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDirection(&self, value: AnimationDirection) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDirection)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn DelayBehavior(&self) -> windows_core::Result<AnimationDelayBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation3>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).DelayBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDelayBehavior(
        &self,
        value: AnimationDelayBehavior,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation3>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDelayBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for QuaternionKeyFrameAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IQuaternionKeyFrameAnimation>();
}
unsafe impl windows_core::Interface for QuaternionKeyFrameAnimation {
    type Vtable = <IQuaternionKeyFrameAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IQuaternionKeyFrameAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for QuaternionKeyFrameAnimation {
    const NAME: &'static str = "Windows.UI.Composition.QuaternionKeyFrameAnimation";
}
unsafe impl Send for QuaternionKeyFrameAnimation {}
unsafe impl Sync for QuaternionKeyFrameAnimation {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarKeyFrameAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ScalarKeyFrameAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    ScalarKeyFrameAnimation,
    IAnimationObject,
    ICompositionAnimationBase,
    KeyFrameAnimation,
    CompositionAnimation,
    CompositionObject
);
impl ScalarKeyFrameAnimation {
    pub(crate) fn ClearAllParameters(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearAllParameters)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub(crate) fn ClearParameter(&self, key: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
            )
            .ok()
        }
    }
    pub(crate) fn SetColorParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Color,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetColorParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix3x2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix3x2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix4x4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix4x4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetQuaternionParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Quaternion,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetQuaternionParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetReferenceParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScalarParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector3Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector3Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetBooleanParameter(
        &self,
        key: &windows_core::HSTRING,
        value: bool,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBooleanParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Target(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Target)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetTarget(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTarget)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn DelayTime(&self) -> windows_core::Result<windows_time::TimeSpan> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).DelayTime)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDelayTime(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDelayTime)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Duration(&self) -> windows_core::Result<windows_time::TimeSpan> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Duration)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDuration(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDuration)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IterationBehavior(&self) -> windows_core::Result<AnimationIterationBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IterationBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIterationBehavior(
        &self,
        value: AnimationIterationBehavior,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIterationBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IterationCount(&self) -> windows_core::Result<i32> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IterationCount)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIterationCount(&self, value: i32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIterationCount)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn KeyFrameCount(&self) -> windows_core::Result<i32> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).KeyFrameCount)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn StopBehavior(&self) -> windows_core::Result<AnimationStopBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).StopBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetStopBehavior(&self, value: AnimationStopBehavior) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetStopBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertExpressionKeyFrame(
        &self,
        normalizedprogresskey: f32,
        value: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).InsertExpressionKeyFrame)(
                windows_core::Interface::as_raw(this),
                normalizedprogresskey,
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn InsertExpressionKeyFrameWithEasingFunction<P2>(
        &self,
        normalizedprogresskey: f32,
        value: &windows_core::HSTRING,
        easingfunction: P2,
    ) -> windows_core::Result<()>
    where
        P2: windows_core::Param<CompositionEasingFunction>,
    {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).InsertExpressionKeyFrameWithEasingFunction)(
                windows_core::Interface::as_raw(this),
                normalizedprogresskey,
                core::mem::transmute_copy(value),
                easingfunction.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn Direction(&self) -> windows_core::Result<AnimationDirection> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Direction)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDirection(&self, value: AnimationDirection) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDirection)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn DelayBehavior(&self) -> windows_core::Result<AnimationDelayBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation3>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).DelayBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDelayBehavior(
        &self,
        value: AnimationDelayBehavior,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation3>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDelayBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertKeyFrame(
        &self,
        normalizedprogresskey: f32,
        value: f32,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).InsertKeyFrame)(
                windows_core::Interface::as_raw(self),
                normalizedprogresskey,
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertKeyFrameWithEasingFunction<P2>(
        &self,
        normalizedprogresskey: f32,
        value: f32,
        easingfunction: P2,
    ) -> windows_core::Result<()>
    where
        P2: windows_core::Param<CompositionEasingFunction>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).InsertKeyFrameWithEasingFunction)(
                windows_core::Interface::as_raw(self),
                normalizedprogresskey,
                value,
                easingfunction.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for ScalarKeyFrameAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IScalarKeyFrameAnimation>();
}
unsafe impl windows_core::Interface for ScalarKeyFrameAnimation {
    type Vtable = <IScalarKeyFrameAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IScalarKeyFrameAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for ScalarKeyFrameAnimation {
    const NAME: &'static str = "Windows.UI.Composition.ScalarKeyFrameAnimation";
}
unsafe impl Send for ScalarKeyFrameAnimation {}
unsafe impl Sync for ScalarKeyFrameAnimation {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarNaturalMotionAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ScalarNaturalMotionAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    ScalarNaturalMotionAnimation,
    NaturalMotionAnimation,
    CompositionAnimation,
    CompositionObject
);
impl ScalarNaturalMotionAnimation {
    pub(crate) fn ClearAllParameters(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearAllParameters)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub(crate) fn ClearParameter(&self, key: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
            )
            .ok()
        }
    }
    pub(crate) fn SetColorParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Color,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetColorParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix3x2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix3x2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix4x4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix4x4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetQuaternionParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Quaternion,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetQuaternionParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetReferenceParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScalarParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector3Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector3Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetBooleanParameter(
        &self,
        key: &windows_core::HSTRING,
        value: bool,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBooleanParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Target(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Target)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetTarget(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTarget)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for ScalarNaturalMotionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IScalarNaturalMotionAnimation>();
}
unsafe impl windows_core::Interface for ScalarNaturalMotionAnimation {
    type Vtable = <IScalarNaturalMotionAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IScalarNaturalMotionAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for ScalarNaturalMotionAnimation {
    const NAME: &'static str = "Windows.UI.Composition.ScalarNaturalMotionAnimation";
}
unsafe impl Send for ScalarNaturalMotionAnimation {}
unsafe impl Sync for ScalarNaturalMotionAnimation {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SineEasingFunction(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    SineEasingFunction,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    SineEasingFunction,
    CompositionEasingFunction,
    CompositionObject
);
impl SineEasingFunction {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for SineEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ISineEasingFunction>();
}
unsafe impl windows_core::Interface for SineEasingFunction {
    type Vtable = <ISineEasingFunction as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ISineEasingFunction as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for SineEasingFunction {
    const NAME: &'static str = "Windows.UI.Composition.SineEasingFunction";
}
unsafe impl Send for SineEasingFunction {}
unsafe impl Sync for SineEasingFunction {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpotLight(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    SpotLight,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(SpotLight, CompositionLight, CompositionObject);
impl SpotLight {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for SpotLight {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ISpotLight>();
}
unsafe impl windows_core::Interface for SpotLight {
    type Vtable = <ISpotLight as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ISpotLight as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for SpotLight {
    const NAME: &'static str = "Windows.UI.Composition.SpotLight";
}
unsafe impl Send for SpotLight {}
unsafe impl Sync for SpotLight {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpringScalarNaturalMotionAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    SpringScalarNaturalMotionAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    SpringScalarNaturalMotionAnimation,
    ScalarNaturalMotionAnimation,
    NaturalMotionAnimation,
    CompositionAnimation,
    CompositionObject
);
impl SpringScalarNaturalMotionAnimation {
    pub(crate) fn ClearAllParameters(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearAllParameters)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub(crate) fn ClearParameter(&self, key: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
            )
            .ok()
        }
    }
    pub(crate) fn SetColorParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Color,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetColorParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix3x2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix3x2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix4x4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix4x4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetQuaternionParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Quaternion,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetQuaternionParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetReferenceParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScalarParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector3Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector3Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetBooleanParameter(
        &self,
        key: &windows_core::HSTRING,
        value: bool,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBooleanParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Target(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Target)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetTarget(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTarget)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for SpringScalarNaturalMotionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ISpringScalarNaturalMotionAnimation>();
}
unsafe impl windows_core::Interface for SpringScalarNaturalMotionAnimation {
    type Vtable = <ISpringScalarNaturalMotionAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <ISpringScalarNaturalMotionAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for SpringScalarNaturalMotionAnimation {
    const NAME: &'static str = "Windows.UI.Composition.SpringScalarNaturalMotionAnimation";
}
unsafe impl Send for SpringScalarNaturalMotionAnimation {}
unsafe impl Sync for SpringScalarNaturalMotionAnimation {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpringVector2NaturalMotionAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    SpringVector2NaturalMotionAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    SpringVector2NaturalMotionAnimation,
    Vector2NaturalMotionAnimation,
    NaturalMotionAnimation,
    CompositionAnimation,
    CompositionObject
);
impl SpringVector2NaturalMotionAnimation {
    pub(crate) fn ClearAllParameters(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearAllParameters)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub(crate) fn ClearParameter(&self, key: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
            )
            .ok()
        }
    }
    pub(crate) fn SetColorParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Color,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetColorParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix3x2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix3x2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix4x4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix4x4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetQuaternionParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Quaternion,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetQuaternionParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetReferenceParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScalarParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector3Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector3Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetBooleanParameter(
        &self,
        key: &windows_core::HSTRING,
        value: bool,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBooleanParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Target(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Target)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetTarget(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTarget)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for SpringVector2NaturalMotionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ISpringVector2NaturalMotionAnimation>();
}
unsafe impl windows_core::Interface for SpringVector2NaturalMotionAnimation {
    type Vtable = <ISpringVector2NaturalMotionAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <ISpringVector2NaturalMotionAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for SpringVector2NaturalMotionAnimation {
    const NAME: &'static str = "Windows.UI.Composition.SpringVector2NaturalMotionAnimation";
}
unsafe impl Send for SpringVector2NaturalMotionAnimation {}
unsafe impl Sync for SpringVector2NaturalMotionAnimation {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpringVector3NaturalMotionAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    SpringVector3NaturalMotionAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    SpringVector3NaturalMotionAnimation,
    Vector3NaturalMotionAnimation,
    NaturalMotionAnimation,
    CompositionAnimation,
    CompositionObject
);
impl SpringVector3NaturalMotionAnimation {
    pub(crate) fn ClearAllParameters(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearAllParameters)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub(crate) fn ClearParameter(&self, key: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
            )
            .ok()
        }
    }
    pub(crate) fn SetColorParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Color,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetColorParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix3x2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix3x2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix4x4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix4x4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetQuaternionParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Quaternion,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetQuaternionParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetReferenceParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScalarParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector3Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector3Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetBooleanParameter(
        &self,
        key: &windows_core::HSTRING,
        value: bool,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBooleanParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Target(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Target)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetTarget(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTarget)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for SpringVector3NaturalMotionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ISpringVector3NaturalMotionAnimation>();
}
unsafe impl windows_core::Interface for SpringVector3NaturalMotionAnimation {
    type Vtable = <ISpringVector3NaturalMotionAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <ISpringVector3NaturalMotionAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for SpringVector3NaturalMotionAnimation {
    const NAME: &'static str = "Windows.UI.Composition.SpringVector3NaturalMotionAnimation";
}
unsafe impl Send for SpringVector3NaturalMotionAnimation {}
unsafe impl Sync for SpringVector3NaturalMotionAnimation {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpriteVisual(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    SpriteVisual,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    SpriteVisual,
    IAnimationObject,
    ContainerVisual,
    Visual,
    CompositionObject
);
impl SpriteVisual {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn Children(&self) -> windows_core::Result<VisualCollection> {
        let this = &windows_core::Interface::cast::<IContainerVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Children)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Brush(&self) -> windows_core::Result<CompositionBrush> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Brush)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetBrush<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<CompositionBrush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetBrush)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn AnchorPoint(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).AnchorPoint)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetAnchorPoint(
        &self,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetAnchorPoint)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn BackfaceVisibility(&self) -> windows_core::Result<CompositionBackfaceVisibility> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).BackfaceVisibility)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetBackfaceVisibility(
        &self,
        value: CompositionBackfaceVisibility,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBackfaceVisibility)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn BorderMode(&self) -> windows_core::Result<CompositionBorderMode> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).BorderMode)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetBorderMode(&self, value: CompositionBorderMode) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBorderMode)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn CenterPoint(&self) -> windows_core::Result<windows_numerics::Vector3> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CenterPoint)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetCenterPoint(
        &self,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetCenterPoint)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Clip(&self) -> windows_core::Result<CompositionClip> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Clip)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetClip<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<CompositionClip>,
    {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetClip)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn CompositeMode(&self) -> windows_core::Result<CompositionCompositeMode> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CompositeMode)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetCompositeMode(
        &self,
        value: CompositionCompositeMode,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetCompositeMode)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IsVisible(&self) -> windows_core::Result<bool> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IsVisible)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIsVisible(&self, value: bool) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIsVisible)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Offset(&self) -> windows_core::Result<windows_numerics::Vector3> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Offset)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetOffset(&self, value: windows_numerics::Vector3) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetOffset)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Opacity(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Opacity)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetOpacity(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetOpacity)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Orientation(&self) -> windows_core::Result<Quaternion> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Orientation)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetOrientation(&self, value: Quaternion) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetOrientation)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Parent(&self) -> windows_core::Result<ContainerVisual> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Parent)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn RotationAngle(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RotationAngle)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAngle(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRotationAngle)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RotationAngleInDegrees(&self) -> windows_core::Result<f32> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RotationAngleInDegrees)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAngleInDegrees(&self, value: f32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRotationAngleInDegrees)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RotationAxis(&self) -> windows_core::Result<windows_numerics::Vector3> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RotationAxis)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAxis(
        &self,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRotationAxis)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Scale(&self) -> windows_core::Result<windows_numerics::Vector3> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Scale)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetScale(&self, value: windows_numerics::Vector3) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScale)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Size(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Size)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetSize(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetSize)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn TransformMatrix(&self) -> windows_core::Result<windows_numerics::Matrix4x4> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).TransformMatrix)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetTransformMatrix(
        &self,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTransformMatrix)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn ParentForTransform(&self) -> windows_core::Result<Visual> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ParentForTransform)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetParentForTransform<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Visual>,
    {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetParentForTransform)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn RelativeOffsetAdjustment(
        &self,
    ) -> windows_core::Result<windows_numerics::Vector3> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RelativeOffsetAdjustment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRelativeOffsetAdjustment(
        &self,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRelativeOffsetAdjustment)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RelativeSizeAdjustment(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RelativeSizeAdjustment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRelativeSizeAdjustment(
        &self,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRelativeSizeAdjustment)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for SpriteVisual {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ISpriteVisual>();
}
unsafe impl windows_core::Interface for SpriteVisual {
    type Vtable = <ISpriteVisual as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ISpriteVisual as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for SpriteVisual {
    const NAME: &'static str = "Windows.UI.Composition.SpriteVisual";
}
unsafe impl Send for SpriteVisual {}
unsafe impl Sync for SpriteVisual {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepEasingFunction(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    StepEasingFunction,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    StepEasingFunction,
    CompositionEasingFunction,
    CompositionObject
);
impl StepEasingFunction {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for StepEasingFunction {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IStepEasingFunction>();
}
unsafe impl windows_core::Interface for StepEasingFunction {
    type Vtable = <IStepEasingFunction as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IStepEasingFunction as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for StepEasingFunction {
    const NAME: &'static str = "Windows.UI.Composition.StepEasingFunction";
}
unsafe impl Send for StepEasingFunction {}
unsafe impl Sync for StepEasingFunction {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedEventHandler<TSender, TResult>(
    windows_core::IUnknown,
    core::marker::PhantomData<TSender>,
    core::marker::PhantomData<TResult>,
)
where
    TSender: windows_core::RuntimeType + 'static,
    TResult: windows_core::RuntimeType + 'static;
unsafe impl<
    TSender: windows_core::RuntimeType + 'static,
    TResult: windows_core::RuntimeType + 'static,
> windows_core::Interface for TypedEventHandler<TSender, TResult>
{
    type Vtable = TypedEventHandler_Vtbl<TSender, TResult>;
    const IID: windows_core::GUID =
        windows_core::GUID::from_signature(<Self as windows_core::RuntimeType>::SIGNATURE);
}
impl<TSender: windows_core::RuntimeType + 'static, TResult: windows_core::RuntimeType + 'static>
    windows_core::RuntimeType for TypedEventHandler<TSender, TResult>
{
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::new()
        .push_slice(b"pinterface({9de1c534-6ae1-11e0-84e1-18a905bcc53f}")
        .push_slice(b";")
        .push_other(TSender::SIGNATURE)
        .push_slice(b";")
        .push_other(TResult::SIGNATURE)
        .push_slice(b")");
}
impl<TSender: windows_core::RuntimeType + 'static, TResult: windows_core::RuntimeType + 'static>
    TypedEventHandler<TSender, TResult>
{
    pub(crate) fn new<
        F: Fn(windows_core::Ref<TSender>, windows_core::Ref<TResult>) -> windows_core::Result<()>
            + Send
            + 'static,
    >(
        invoke: F,
    ) -> Self {
        let com = windows_core::imp::DelegateBox::<Self, F>::new(
            &TypedEventHandlerBox::<TSender, TResult, F>::VTABLE,
            invoke,
        );
        unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
    }
    pub(crate) fn Invoke<P0, P1>(&self, sender: P0, args: P1) -> windows_core::Result<()>
    where
        P0: windows_core::Param<TSender>,
        P1: windows_core::Param<TResult>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Invoke)(
                windows_core::Interface::as_raw(self),
                sender.param().abi(),
                args.param().abi(),
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct TypedEventHandler_Vtbl<TSender, TResult>
where
    TSender: windows_core::RuntimeType + 'static,
    TResult: windows_core::RuntimeType + 'static,
{
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: windows_core::imp::AbiType<TSender>,
        args: windows_core::imp::AbiType<TResult>,
    ) -> windows_core::HRESULT,
    TSender: core::marker::PhantomData<TSender>,
    TResult: core::marker::PhantomData<TResult>,
}
struct TypedEventHandlerBox<
    TSender,
    TResult,
    F: Fn(windows_core::Ref<TSender>, windows_core::Ref<TResult>) -> windows_core::Result<()>
        + Send
        + 'static,
>(core::marker::PhantomData<(TSender, TResult, fn() -> F)>)
where
    TSender: windows_core::RuntimeType + 'static,
    TResult: windows_core::RuntimeType + 'static;
impl<
    TSender: windows_core::RuntimeType + 'static,
    TResult: windows_core::RuntimeType + 'static,
    F: Fn(windows_core::Ref<TSender>, windows_core::Ref<TResult>) -> windows_core::Result<()>
        + Send
        + 'static,
> TypedEventHandlerBox<TSender, TResult, F>
{
    const VTABLE : TypedEventHandler_Vtbl < TSender , TResult , > = TypedEventHandler_Vtbl::< TSender , TResult , > { base__ : windows_core::IUnknown_Vtbl { QueryInterface : windows_core::imp::DelegateBox::< TypedEventHandler < TSender , TResult > , F >::QueryInterface , AddRef : windows_core::imp::DelegateBox::< TypedEventHandler < TSender , TResult > , F >::AddRef , Release : windows_core::imp::DelegateBox::< TypedEventHandler < TSender , TResult > , F >::Release , } , Invoke : Self::Invoke , TSender : core::marker::PhantomData::< TSender > , TResult : core::marker::PhantomData::< TResult > } ;
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: windows_core::imp::AbiType<TSender>,
        args: windows_core::imp::AbiType<TResult>,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<TypedEventHandler<TSender, TResult>, F>);
            (this.invoke)(
                core::mem::transmute_copy(&sender),
                core::mem::transmute_copy(&args),
            )
            .into()
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vector2KeyFrameAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    Vector2KeyFrameAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    Vector2KeyFrameAnimation,
    IAnimationObject,
    ICompositionAnimationBase,
    KeyFrameAnimation,
    CompositionAnimation,
    CompositionObject
);
impl Vector2KeyFrameAnimation {
    pub(crate) fn ClearAllParameters(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearAllParameters)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub(crate) fn ClearParameter(&self, key: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
            )
            .ok()
        }
    }
    pub(crate) fn SetColorParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Color,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetColorParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix3x2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix3x2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix4x4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix4x4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetQuaternionParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Quaternion,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetQuaternionParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetReferenceParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScalarParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector3Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector3Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetBooleanParameter(
        &self,
        key: &windows_core::HSTRING,
        value: bool,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBooleanParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Target(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Target)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetTarget(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTarget)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn DelayTime(&self) -> windows_core::Result<windows_time::TimeSpan> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).DelayTime)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDelayTime(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDelayTime)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Duration(&self) -> windows_core::Result<windows_time::TimeSpan> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Duration)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDuration(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDuration)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IterationBehavior(&self) -> windows_core::Result<AnimationIterationBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IterationBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIterationBehavior(
        &self,
        value: AnimationIterationBehavior,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIterationBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IterationCount(&self) -> windows_core::Result<i32> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IterationCount)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIterationCount(&self, value: i32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIterationCount)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn KeyFrameCount(&self) -> windows_core::Result<i32> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).KeyFrameCount)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn StopBehavior(&self) -> windows_core::Result<AnimationStopBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).StopBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetStopBehavior(&self, value: AnimationStopBehavior) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetStopBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertExpressionKeyFrame(
        &self,
        normalizedprogresskey: f32,
        value: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).InsertExpressionKeyFrame)(
                windows_core::Interface::as_raw(this),
                normalizedprogresskey,
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn InsertExpressionKeyFrameWithEasingFunction<P2>(
        &self,
        normalizedprogresskey: f32,
        value: &windows_core::HSTRING,
        easingfunction: P2,
    ) -> windows_core::Result<()>
    where
        P2: windows_core::Param<CompositionEasingFunction>,
    {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).InsertExpressionKeyFrameWithEasingFunction)(
                windows_core::Interface::as_raw(this),
                normalizedprogresskey,
                core::mem::transmute_copy(value),
                easingfunction.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn Direction(&self) -> windows_core::Result<AnimationDirection> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Direction)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDirection(&self, value: AnimationDirection) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDirection)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn DelayBehavior(&self) -> windows_core::Result<AnimationDelayBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation3>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).DelayBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDelayBehavior(
        &self,
        value: AnimationDelayBehavior,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation3>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDelayBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertKeyFrame(
        &self,
        normalizedprogresskey: f32,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).InsertKeyFrame)(
                windows_core::Interface::as_raw(self),
                normalizedprogresskey,
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertKeyFrameWithEasingFunction<P2>(
        &self,
        normalizedprogresskey: f32,
        value: windows_numerics::Vector2,
        easingfunction: P2,
    ) -> windows_core::Result<()>
    where
        P2: windows_core::Param<CompositionEasingFunction>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).InsertKeyFrameWithEasingFunction)(
                windows_core::Interface::as_raw(self),
                normalizedprogresskey,
                value,
                easingfunction.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for Vector2KeyFrameAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVector2KeyFrameAnimation>();
}
unsafe impl windows_core::Interface for Vector2KeyFrameAnimation {
    type Vtable = <IVector2KeyFrameAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IVector2KeyFrameAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for Vector2KeyFrameAnimation {
    const NAME: &'static str = "Windows.UI.Composition.Vector2KeyFrameAnimation";
}
unsafe impl Send for Vector2KeyFrameAnimation {}
unsafe impl Sync for Vector2KeyFrameAnimation {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vector2NaturalMotionAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    Vector2NaturalMotionAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    Vector2NaturalMotionAnimation,
    NaturalMotionAnimation,
    CompositionAnimation,
    CompositionObject
);
impl Vector2NaturalMotionAnimation {
    pub(crate) fn ClearAllParameters(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearAllParameters)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub(crate) fn ClearParameter(&self, key: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
            )
            .ok()
        }
    }
    pub(crate) fn SetColorParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Color,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetColorParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix3x2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix3x2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix4x4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix4x4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetQuaternionParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Quaternion,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetQuaternionParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetReferenceParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScalarParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector3Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector3Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetBooleanParameter(
        &self,
        key: &windows_core::HSTRING,
        value: bool,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBooleanParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Target(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Target)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetTarget(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTarget)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for Vector2NaturalMotionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVector2NaturalMotionAnimation>();
}
unsafe impl windows_core::Interface for Vector2NaturalMotionAnimation {
    type Vtable = <IVector2NaturalMotionAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <IVector2NaturalMotionAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for Vector2NaturalMotionAnimation {
    const NAME: &'static str = "Windows.UI.Composition.Vector2NaturalMotionAnimation";
}
unsafe impl Send for Vector2NaturalMotionAnimation {}
unsafe impl Sync for Vector2NaturalMotionAnimation {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vector3KeyFrameAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    Vector3KeyFrameAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    Vector3KeyFrameAnimation,
    IAnimationObject,
    ICompositionAnimationBase,
    KeyFrameAnimation,
    CompositionAnimation,
    CompositionObject
);
impl Vector3KeyFrameAnimation {
    pub(crate) fn ClearAllParameters(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearAllParameters)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub(crate) fn ClearParameter(&self, key: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
            )
            .ok()
        }
    }
    pub(crate) fn SetColorParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Color,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetColorParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix3x2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix3x2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix4x4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix4x4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetQuaternionParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Quaternion,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetQuaternionParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetReferenceParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScalarParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector3Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector3Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetBooleanParameter(
        &self,
        key: &windows_core::HSTRING,
        value: bool,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBooleanParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Target(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Target)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetTarget(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTarget)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn DelayTime(&self) -> windows_core::Result<windows_time::TimeSpan> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).DelayTime)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDelayTime(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDelayTime)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Duration(&self) -> windows_core::Result<windows_time::TimeSpan> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Duration)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDuration(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDuration)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IterationBehavior(&self) -> windows_core::Result<AnimationIterationBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IterationBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIterationBehavior(
        &self,
        value: AnimationIterationBehavior,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIterationBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IterationCount(&self) -> windows_core::Result<i32> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IterationCount)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIterationCount(&self, value: i32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIterationCount)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn KeyFrameCount(&self) -> windows_core::Result<i32> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).KeyFrameCount)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn StopBehavior(&self) -> windows_core::Result<AnimationStopBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).StopBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetStopBehavior(&self, value: AnimationStopBehavior) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetStopBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertExpressionKeyFrame(
        &self,
        normalizedprogresskey: f32,
        value: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).InsertExpressionKeyFrame)(
                windows_core::Interface::as_raw(this),
                normalizedprogresskey,
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn InsertExpressionKeyFrameWithEasingFunction<P2>(
        &self,
        normalizedprogresskey: f32,
        value: &windows_core::HSTRING,
        easingfunction: P2,
    ) -> windows_core::Result<()>
    where
        P2: windows_core::Param<CompositionEasingFunction>,
    {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).InsertExpressionKeyFrameWithEasingFunction)(
                windows_core::Interface::as_raw(this),
                normalizedprogresskey,
                core::mem::transmute_copy(value),
                easingfunction.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn Direction(&self) -> windows_core::Result<AnimationDirection> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Direction)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDirection(&self, value: AnimationDirection) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDirection)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn DelayBehavior(&self) -> windows_core::Result<AnimationDelayBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation3>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).DelayBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDelayBehavior(
        &self,
        value: AnimationDelayBehavior,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation3>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDelayBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertKeyFrame(
        &self,
        normalizedprogresskey: f32,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).InsertKeyFrame)(
                windows_core::Interface::as_raw(self),
                normalizedprogresskey,
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertKeyFrameWithEasingFunction<P2>(
        &self,
        normalizedprogresskey: f32,
        value: windows_numerics::Vector3,
        easingfunction: P2,
    ) -> windows_core::Result<()>
    where
        P2: windows_core::Param<CompositionEasingFunction>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).InsertKeyFrameWithEasingFunction)(
                windows_core::Interface::as_raw(self),
                normalizedprogresskey,
                value,
                easingfunction.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for Vector3KeyFrameAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVector3KeyFrameAnimation>();
}
unsafe impl windows_core::Interface for Vector3KeyFrameAnimation {
    type Vtable = <IVector3KeyFrameAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IVector3KeyFrameAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for Vector3KeyFrameAnimation {
    const NAME: &'static str = "Windows.UI.Composition.Vector3KeyFrameAnimation";
}
unsafe impl Send for Vector3KeyFrameAnimation {}
unsafe impl Sync for Vector3KeyFrameAnimation {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vector3NaturalMotionAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    Vector3NaturalMotionAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    Vector3NaturalMotionAnimation,
    NaturalMotionAnimation,
    CompositionAnimation,
    CompositionObject
);
impl Vector3NaturalMotionAnimation {
    pub(crate) fn ClearAllParameters(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearAllParameters)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub(crate) fn ClearParameter(&self, key: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
            )
            .ok()
        }
    }
    pub(crate) fn SetColorParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Color,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetColorParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix3x2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix3x2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix4x4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix4x4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetQuaternionParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Quaternion,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetQuaternionParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetReferenceParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScalarParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector3Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector3Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetBooleanParameter(
        &self,
        key: &windows_core::HSTRING,
        value: bool,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBooleanParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Target(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Target)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetTarget(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTarget)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for Vector3NaturalMotionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVector3NaturalMotionAnimation>();
}
unsafe impl windows_core::Interface for Vector3NaturalMotionAnimation {
    type Vtable = <IVector3NaturalMotionAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <IVector3NaturalMotionAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for Vector3NaturalMotionAnimation {
    const NAME: &'static str = "Windows.UI.Composition.Vector3NaturalMotionAnimation";
}
unsafe impl Send for Vector3NaturalMotionAnimation {}
unsafe impl Sync for Vector3NaturalMotionAnimation {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vector4KeyFrameAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    Vector4KeyFrameAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    Vector4KeyFrameAnimation,
    KeyFrameAnimation,
    CompositionAnimation,
    CompositionObject
);
impl Vector4KeyFrameAnimation {
    pub(crate) fn ClearAllParameters(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearAllParameters)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub(crate) fn ClearParameter(&self, key: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).ClearParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
            )
            .ok()
        }
    }
    pub(crate) fn SetColorParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Color,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetColorParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix3x2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix3x2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix3x2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetMatrix4x4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMatrix4x4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetQuaternionParameter(
        &self,
        key: &windows_core::HSTRING,
        value: Quaternion,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetQuaternionParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetReferenceParameter<P1>(
        &self,
        key: &windows_core::HSTRING,
        compositionobject: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionObject>,
    {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetReferenceParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                compositionobject.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn SetScalarParameter(
        &self,
        key: &windows_core::HSTRING,
        value: f32,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetScalarParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector2Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector2Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector3Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector3Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetVector4Parameter(
        &self,
        key: &windows_core::HSTRING,
        value: windows_numerics::Vector4,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetVector4Parameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn SetBooleanParameter(
        &self,
        key: &windows_core::HSTRING,
        value: bool,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetBooleanParameter)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(key),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Target(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Target)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetTarget(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetTarget)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn DelayTime(&self) -> windows_core::Result<windows_time::TimeSpan> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).DelayTime)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDelayTime(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDelayTime)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Duration(&self) -> windows_core::Result<windows_time::TimeSpan> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Duration)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDuration(&self, value: windows_time::TimeSpan) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDuration)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IterationBehavior(&self) -> windows_core::Result<AnimationIterationBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IterationBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIterationBehavior(
        &self,
        value: AnimationIterationBehavior,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIterationBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IterationCount(&self) -> windows_core::Result<i32> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IterationCount)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIterationCount(&self, value: i32) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIterationCount)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn KeyFrameCount(&self) -> windows_core::Result<i32> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).KeyFrameCount)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn StopBehavior(&self) -> windows_core::Result<AnimationStopBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).StopBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetStopBehavior(&self, value: AnimationStopBehavior) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetStopBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn InsertExpressionKeyFrame(
        &self,
        normalizedprogresskey: f32,
        value: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).InsertExpressionKeyFrame)(
                windows_core::Interface::as_raw(this),
                normalizedprogresskey,
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn InsertExpressionKeyFrameWithEasingFunction<P2>(
        &self,
        normalizedprogresskey: f32,
        value: &windows_core::HSTRING,
        easingfunction: P2,
    ) -> windows_core::Result<()>
    where
        P2: windows_core::Param<CompositionEasingFunction>,
    {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).InsertExpressionKeyFrameWithEasingFunction)(
                windows_core::Interface::as_raw(this),
                normalizedprogresskey,
                core::mem::transmute_copy(value),
                easingfunction.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn Direction(&self) -> windows_core::Result<AnimationDirection> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Direction)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDirection(&self, value: AnimationDirection) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDirection)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn DelayBehavior(&self) -> windows_core::Result<AnimationDelayBehavior> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation3>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).DelayBehavior)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetDelayBehavior(
        &self,
        value: AnimationDelayBehavior,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IKeyFrameAnimation3>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetDelayBehavior)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for Vector4KeyFrameAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVector4KeyFrameAnimation>();
}
unsafe impl windows_core::Interface for Vector4KeyFrameAnimation {
    type Vtable = <IVector4KeyFrameAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IVector4KeyFrameAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for Vector4KeyFrameAnimation {
    const NAME: &'static str = "Windows.UI.Composition.Vector4KeyFrameAnimation";
}
unsafe impl Send for Vector4KeyFrameAnimation {}
unsafe impl Sync for Vector4KeyFrameAnimation {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Visual(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Visual, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(Visual, IAnimationObject, CompositionObject);
impl Visual {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn AnchorPoint(&self) -> windows_core::Result<windows_numerics::Vector2> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).AnchorPoint)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetAnchorPoint(
        &self,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetAnchorPoint)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn BackfaceVisibility(&self) -> windows_core::Result<CompositionBackfaceVisibility> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).BackfaceVisibility)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetBackfaceVisibility(
        &self,
        value: CompositionBackfaceVisibility,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetBackfaceVisibility)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn BorderMode(&self) -> windows_core::Result<CompositionBorderMode> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).BorderMode)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetBorderMode(&self, value: CompositionBorderMode) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetBorderMode)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn CenterPoint(&self) -> windows_core::Result<windows_numerics::Vector3> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CenterPoint)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetCenterPoint(
        &self,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetCenterPoint)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Clip(&self) -> windows_core::Result<CompositionClip> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Clip)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetClip<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<CompositionClip>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetClip)(
                windows_core::Interface::as_raw(self),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn CompositeMode(&self) -> windows_core::Result<CompositionCompositeMode> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CompositeMode)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetCompositeMode(
        &self,
        value: CompositionCompositeMode,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetCompositeMode)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IsVisible(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IsVisible)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIsVisible(&self, value: bool) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetIsVisible)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Offset(&self) -> windows_core::Result<windows_numerics::Vector3> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Offset)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetOffset(&self, value: windows_numerics::Vector3) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetOffset)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Opacity(&self) -> windows_core::Result<f32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Opacity)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetOpacity(&self, value: f32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetOpacity)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Orientation(&self) -> windows_core::Result<Quaternion> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Orientation)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetOrientation(&self, value: Quaternion) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetOrientation)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Parent(&self) -> windows_core::Result<ContainerVisual> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Parent)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn RotationAngle(&self) -> windows_core::Result<f32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RotationAngle)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAngle(&self, value: f32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetRotationAngle)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RotationAngleInDegrees(&self) -> windows_core::Result<f32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RotationAngleInDegrees)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAngleInDegrees(&self, value: f32) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetRotationAngleInDegrees)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RotationAxis(&self) -> windows_core::Result<windows_numerics::Vector3> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RotationAxis)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRotationAxis(
        &self,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetRotationAxis)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Scale(&self) -> windows_core::Result<windows_numerics::Vector3> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Scale)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetScale(&self, value: windows_numerics::Vector3) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetScale)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn Size(&self) -> windows_core::Result<windows_numerics::Vector2> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Size)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetSize(&self, value: windows_numerics::Vector2) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetSize)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn TransformMatrix(&self) -> windows_core::Result<windows_numerics::Matrix4x4> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TransformMatrix)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetTransformMatrix(
        &self,
        value: windows_numerics::Matrix4x4,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetTransformMatrix)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn ParentForTransform(&self) -> windows_core::Result<Self> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ParentForTransform)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetParentForTransform<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Self>,
    {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetParentForTransform)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn RelativeOffsetAdjustment(
        &self,
    ) -> windows_core::Result<windows_numerics::Vector3> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RelativeOffsetAdjustment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRelativeOffsetAdjustment(
        &self,
        value: windows_numerics::Vector3,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRelativeOffsetAdjustment)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn RelativeSizeAdjustment(&self) -> windows_core::Result<windows_numerics::Vector2> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RelativeSizeAdjustment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetRelativeSizeAdjustment(
        &self,
        value: windows_numerics::Vector2,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IVisual2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetRelativeSizeAdjustment)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for Visual {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVisual>();
}
unsafe impl windows_core::Interface for Visual {
    type Vtable = <IVisual as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IVisual as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for Visual {
    const NAME: &'static str = "Windows.UI.Composition.Visual";
}
unsafe impl Send for Visual {}
unsafe impl Sync for Visual {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisualCollection(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    VisualCollection,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    VisualCollection,
    IAnimationObject,
    IIterable<Visual>,
    CompositionObject
);
impl VisualCollection {
    pub(crate) fn Compositor(&self) -> windows_core::Result<Compositor> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Compositor)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Dispatcher(&self) -> windows_core::Result<CoreDispatcher> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Dispatcher)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Properties(&self) -> windows_core::Result<CompositionPropertySet> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Properties)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn StartAnimation<P1>(
        &self,
        propertyname: &windows_core::HSTRING,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P1: windows_core::Param<CompositionAnimation>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimation(
        &self,
        propertyname: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimation)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(propertyname),
            )
            .ok()
        }
    }
    pub(crate) fn Comment(&self) -> windows_core::Result<windows_core::HSTRING> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Comment)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn SetComment(&self, value: &windows_core::HSTRING) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetComment)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(value),
            )
            .ok()
        }
    }
    pub(crate) fn ImplicitAnimations(&self) -> windows_core::Result<ImplicitAnimationCollection> {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn SetImplicitAnimations<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ImplicitAnimationCollection>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetImplicitAnimations)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StartAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn StopAnimationGroup<P0>(&self, value: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<ICompositionAnimationBase>,
    {
        let this = &windows_core::Interface::cast::<ICompositionObject2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StopAnimationGroup)(
                windows_core::Interface::as_raw(this),
                value.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn First(&self) -> windows_core::Result<windows_collections::IIterator<Visual>> {
        let this = &windows_core::Interface::cast::<IIterable<Visual>>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).First)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Count(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Count)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn InsertAbove<P0, P1>(&self, newchild: P0, sibling: P1) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Visual>,
        P1: windows_core::Param<Visual>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).InsertAbove)(
                windows_core::Interface::as_raw(self),
                newchild.param().abi(),
                sibling.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn InsertAtBottom<P0>(&self, newchild: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Visual>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).InsertAtBottom)(
                windows_core::Interface::as_raw(self),
                newchild.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn InsertAtTop<P0>(&self, newchild: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Visual>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).InsertAtTop)(
                windows_core::Interface::as_raw(self),
                newchild.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn InsertBelow<P0, P1>(&self, newchild: P0, sibling: P1) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Visual>,
        P1: windows_core::Param<Visual>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).InsertBelow)(
                windows_core::Interface::as_raw(self),
                newchild.param().abi(),
                sibling.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn Remove<P0>(&self, child: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<Visual>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Remove)(
                windows_core::Interface::as_raw(self),
                child.param().abi(),
            )
            .ok()
        }
    }
    pub(crate) fn RemoveAll(&self) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).RemoveAll)(windows_core::Interface::as_raw(self))
                .ok()
        }
    }
}
impl windows_core::RuntimeType for VisualCollection {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVisualCollection>();
}
unsafe impl windows_core::Interface for VisualCollection {
    type Vtable = <IVisualCollection as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IVisualCollection as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for VisualCollection {
    const NAME: &'static str = "Windows.UI.Composition.VisualCollection";
}
unsafe impl Send for VisualCollection {}
unsafe impl Sync for VisualCollection {}
impl IntoIterator for VisualCollection {
    type Item = Visual;
    type IntoIter = windows_collections::BufferedIterator<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(&self)
    }
}
impl IntoIterator for &VisualCollection {
    type Item = Visual;
    type IntoIter = windows_collections::BufferedIterator<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        windows_collections::BufferedIterator::new(self.First().unwrap())
    }
}
