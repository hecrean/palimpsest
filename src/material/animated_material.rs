use bitflags::bitflags;

use crate::plugins::mouse::Mouse;
use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::system::{
        lifetimeless::{Read, SRes},
        SystemParamItem,
    },
    input::{mouse::MouseButtonInput, ButtonState},
    pbr::{
        DrawMesh, MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup,
        SetMeshViewBindGroup,
    },
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        mesh::MeshVertexBufferLayout,
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        view::{ComputedVisibility, ExtractedView, Msaa, Visibility},
        RenderApp, RenderStage,
    },
};

pub fn setup_animated_cubes(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    // cube
    commands.spawn().insert_bundle((
        meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        Transform::from_xyz(0.0, 0.5, 0.0),
        GlobalTransform::default(),
        AnimatedMaterial,
        Visibility::default(),
        ComputedVisibility::default(),
    ));
}

#[derive(Component)]
pub struct AnimatedMaterial;

pub struct AnimatedMaterialPlugin;

impl Plugin for AnimatedMaterialPlugin {
    fn build(&self, app: &mut App) {
        let render_device = app.world.resource::<RenderDevice>();
        let time_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("time uniform buffer"),
            size: ExtractedTime::SHADER_SIZE.into(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mouse_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("mouse uniform buffer"),
            size: ExtractedMouse::SHADER_SIZE.into(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        app.add_plugin(ExtractComponentPlugin::<AnimatedMaterial>::default())
            .add_plugin(ExtractResourcePlugin::<ExtractedTime>::default())
            .add_plugin(ExtractResourcePlugin::<ExtractedMouse>::default());

        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .insert_resource(TimeMeta {
                buffer: time_buffer,
                bind_group: None,
            })
            .insert_resource(MouseMeta {
                buffer: mouse_buffer,
                bind_group: None,
            })
            .init_resource::<CustomPipeline>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .add_system_to_stage(RenderStage::Prepare, prepare_time)
            .add_system_to_stage(RenderStage::Prepare, prepare_mouse)
            .add_system_to_stage(RenderStage::Queue, queue_custom)
            .add_system_to_stage(RenderStage::Queue, queue_time_bind_group)
            .add_system_to_stage(RenderStage::Queue, queue_mouse_bind_group);
    }
}

impl ExtractComponent for AnimatedMaterial {
    type Query = Read<AnimatedMaterial>;

    type Filter = ();

    fn extract_component(_: bevy::ecs::query::QueryItem<Self::Query>) -> Self {
        AnimatedMaterial
    }
}

// add each entity with a mesh and a `AnimatedMaterial` to every view's `Transparent3d` render phase using the `CustomPipeline`
#[allow(clippy::too_many_arguments)]
fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<CustomPipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<CustomPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    render_meshes: Res<RenderAssets<Mesh>>,
    material_meshes: Query<(Entity, &MeshUniform, &Handle<Mesh>), With<AnimatedMaterial>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_custom = transparent_3d_draw_functions
        .read()
        .get_id::<DrawCustom>()
        .unwrap();

    let key = MeshPipelineKey::from_msaa_samples(msaa.samples)
        | MeshPipelineKey::from_primitive_topology(PrimitiveTopology::TriangleList);

    for (view, mut transparent_phase) in &mut views {
        let rangefinder = view.rangefinder3d();
        for (entity, mesh_uniform, mesh_handle) in &material_meshes {
            if let Some(mesh) = render_meshes.get(mesh_handle) {
                let pipeline = pipelines
                    .specialize(&mut pipeline_cache, &custom_pipeline, key, &mesh.layout)
                    .unwrap();
                transparent_phase.add(Transparent3d {
                    entity,
                    pipeline,
                    draw_function: draw_custom,
                    distance: rangefinder.distance(&mesh_uniform.transform),
                });
            }
        }
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(ShaderType)]
    struct MouseButtonFlag: u32 {
        const LEFT = 0b00000001;
        const RIGHT = 0b00000010;
        const MIDDLE = 0b00000100;
        const OTHER = 0b00001000;
    }
    #[repr(transparent)]
    #[derive(ShaderType)]
    struct ButtonStateFlag: u32 {
        const PRESSED = 0b00000001;
        const RELEASED = 0b00000010;
    }
}

impl From<MouseButton> for MouseButtonFlag {
    fn from(mouse_button: MouseButton) -> Self {
        match mouse_button {
            MouseButton::Left => MouseButtonFlag::LEFT,
            MouseButton::Right => MouseButtonFlag::RIGHT,
            MouseButton::Middle => MouseButtonFlag::MIDDLE,
            MouseButton::Other(v) => MouseButtonFlag::OTHER,
        }
    }
}
impl From<ButtonState> for ButtonStateFlag {
    fn from(button_state: ButtonState) -> Self {
        match button_state {
            ButtonState::Pressed => ButtonStateFlag::PRESSED,
            ButtonState::Released => ButtonStateFlag::RELEASED,
        }
    }
}

#[derive(Resource, ShaderType)]
struct ExtractedMouse {
    // button: MouseButtonFlag,
    // button_state: ButtonStateFlag,
    normalised_device_coordinates: Vec2,
}

impl ExtractResource for ExtractedMouse {
    type Source = Mouse;

    fn extract_resource(mouse: &Self::Source) -> Self {
        ExtractedMouse {
            // button: mouse.button.into(),
            // button_state: mouse.button_state.into(),
            normalised_device_coordinates: mouse.normalised_device_coordinates,
        }
    }
}

#[derive(Resource)]
struct MouseMeta {
    buffer: Buffer,
    bind_group: Option<BindGroup>,
}

fn prepare_mouse(
    mouse: Res<ExtractedMouse>,
    mouse_meta: ResMut<MouseMeta>,
    render_queue: Res<RenderQueue>,
) {
    let normalised_device_coordinates_bytes =
        bevy::core::bytes_of(&mouse.normalised_device_coordinates);

    render_queue.write_buffer(&mouse_meta.buffer, 0, normalised_device_coordinates_bytes);
}

// create a bind group for the time uniform buffer
fn queue_mouse_bind_group(
    render_device: Res<RenderDevice>,
    mut mouse_meta: ResMut<MouseMeta>,
    pipeline: Res<CustomPipeline>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.mouse_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: mouse_meta.buffer.as_entire_binding(),
        }],
    });
    mouse_meta.bind_group = Some(bind_group);
}

#[derive(Resource, ShaderType, Default)]
struct ExtractedTime {
    seconds_since_startup: f32,
}

impl ExtractResource for ExtractedTime {
    type Source = Time;

    fn extract_resource(time: &Self::Source) -> Self {
        ExtractedTime {
            seconds_since_startup: time.seconds_since_startup() as f32,
        }
    }
}

#[derive(Resource)]
struct TimeMeta {
    buffer: Buffer,
    bind_group: Option<BindGroup>,
}

// write the extracted time into the corresponding uniform buffer
fn prepare_time(
    time: Res<ExtractedTime>,
    time_meta: ResMut<TimeMeta>,
    render_queue: Res<RenderQueue>,
) {
    render_queue.write_buffer(
        &time_meta.buffer,
        0,
        bevy::core::cast_slice(&[time.seconds_since_startup]),
    );
}

// create a bind group for the time uniform buffer
fn queue_time_bind_group(
    render_device: Res<RenderDevice>,
    mut time_meta: ResMut<TimeMeta>,
    pipeline: Res<CustomPipeline>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.time_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: time_meta.buffer.as_entire_binding(),
        }],
    });
    time_meta.bind_group = Some(bind_group);
}

#[derive(Resource)]
pub struct CustomPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    time_bind_group_layout: BindGroupLayout,
    mouse_bind_group_layout: BindGroupLayout,
}

impl FromWorld for CustomPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/animate_shader.wgsl");

        let render_device = world.resource::<RenderDevice>();
        let time_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("time bind group"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(ExtractedTime::SHADER_SIZE.into()),
                    },
                    count: None,
                }],
            });
        let mouse_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("mouse bind group"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT | ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(ExtractedMouse::SHADER_SIZE.into()),
                    },
                    count: None,
                }],
            });

        let mesh_pipeline = world.resource::<MeshPipeline>();

        CustomPipeline {
            shader,
            mesh_pipeline: mesh_pipeline.clone(),
            time_bind_group_layout,
            mouse_bind_group_layout,
        }
    }
}

impl SpecializedMeshPipeline for CustomPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.vertex.shader = self.shader.clone();
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        descriptor.layout = Some(vec![
            self.mesh_pipeline.view_layout.clone(),
            self.mesh_pipeline.mesh_layout.clone(),
            self.time_bind_group_layout.clone(),
            self.mouse_bind_group_layout.clone(),
        ]);
        Ok(descriptor)
    }
}

type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetTimeBindGroup<2>,
    SetMouseBindGroup<3>,
    DrawMesh,
);

struct SetTimeBindGroup<const I: usize>;

impl<const I: usize> EntityRenderCommand for SetTimeBindGroup<I> {
    type Param = SRes<TimeMeta>;

    fn render<'w>(
        _view: Entity,
        _item: Entity,
        time_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let time_bind_group = time_meta.into_inner().bind_group.as_ref().unwrap();
        pass.set_bind_group(I, time_bind_group, &[]);

        RenderCommandResult::Success
    }
}

struct SetMouseBindGroup<const I: usize>;

impl<const I: usize> EntityRenderCommand for SetMouseBindGroup<I> {
    type Param = SRes<MouseMeta>;

    fn render<'w>(
        _view: Entity,
        _item: Entity,
        mouse_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mouse_bind_group = mouse_meta.into_inner().bind_group.as_ref().unwrap();
        pass.set_bind_group(I, mouse_bind_group, &[]);

        RenderCommandResult::Success
    }
}
