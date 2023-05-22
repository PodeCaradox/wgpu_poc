
struct Tile //16 bytes in total
{
	 InstanceTransform: vec3<f32>,
	 AtlasCoord: u32,   //x/y for column/row z for image index
};

struct TileStorage {
  tiles: array<Tile>,
};


//=============================================================================
// Compute Shader Functions
//=============================================================================
 fn IsInMapBounds(map_position: vec2<i32>) -> i32 {
	if(map_position.x >= 0 && map_position.y >= 0 && map_position.y < params.map_size.y && map_position.x < params.map_size.x) { return 1; }


	return 0;
}

fn CalculateWorldRowPosition(global_id: vec2<u32>) -> vec2<i32>{
	var map_position = params.start_pos;
    let column = i32(global_id.x);
    var row = i32(global_id.y);

    map_position.x -= row % 2;
    row /= 2;
	map_position.y += row;
	map_position.x -= row;
	return map_position;
}

fn calculate_rows(start: vec2<i32>, mapSizeX: i32) -> i32
{
	var rows: i32;
	if (start.y < start.x)
	{
		rows = mapSizeX - (start.x - start.y);
	}
	else
	{
		rows = mapSizeX + (start.y - start.x);
	}


	return rows;
}

fn get_columns_until_border(index: vec2<i32>) -> i32
{
	if (index.x < index.y)
	{
		return index.x;
	}
	return index.y;
}

//=============================================================================
// Compute Shader
//=============================================================================
struct ComputeParams {
    start_pos: vec2<i32>,
    map_size: vec2<i32>,
    columns: i32,
    rows: i32
};
@group(0) @binding(0)
var<uniform> params: ComputeParams;

@group(1) @binding(0) var<storage, read> all_tiles : TileStorage;
@group(2) @binding(0) var<storage, read_write> visble_tiles_cp : TileStorage;

@compute
@workgroup_size(16, 16, 1)
fn calcvisibility(@builtin(global_invocation_id) global_id: vec3<u32>) {
            var index: vec2<i32> = vec2<i32>(params.start_pos.x, params.start_pos.y);
            let column = i32(global_id.x);
            var row = i32(global_id.y);

            index.x -= row % 2;
            row /= 2;
            index.y += row;
            index.x -= row;
            let actual_row_start = index;
            index.y += column;
            index.x += column;

            if (index.x < 0 || index.y < 0 || index.y >= params.map_size.y || index.x >= params.map_size.x) {
                return;
            }

            var visible_index = 0;

            var start: vec2<i32> = vec2<i32>(params.start_pos.x, params.start_pos.y);
            var outside = 1;
            for (var i: i32 = 0; i < params.columns; i+=1){
                start.x += 1;
                start.y += 1;
                if (start.x >= 0 && start.y >= 0 && start.y < params.map_size.y && start.x < params.map_size.x) {
                    outside = 0;
                    break;
                }
            }

            if (outside == 1) {
                if (params.start_pos.x + params.start_pos.y < params.map_size.x) {
                    var left: vec2<i32> = vec2<i32>(params.start_pos.x - params.rows, params.start_pos.y + params.rows);
                    left.x += left.y;
                    left.y -= left.y;

                    var right_bottom_screen: vec2<i32> = vec2<i32>(params.start_pos.x + params.columns, params.start_pos.y + params.columns);
                    if (right_bottom_screen.x + right_bottom_screen.y > params.map_size.x) {
                        start = vec2<i32>(params.map_size.x - 1, 0);
                    } else {
                        right_bottom_screen.x += right_bottom_screen.y;
                        right_bottom_screen.y -= right_bottom_screen.y;
                        start = right_bottom_screen;
                    }
                    var difference = start.x - left.x;
                    difference += difference % 2;
                    difference /= 2;
                    start.x -= difference;
                    start.y -= difference;
                } else {
                    let to_the_left = params.start_pos.x - params.map_size.x;
                    start = vec2<i32>(params.start_pos.x - to_the_left, params.start_pos.y + to_the_left);
                }
            } else {
                start = vec2<i32>(params.start_pos.x, params.start_pos.y);
            }

            let rows_behind = calculate_rows(index, params.map_size.x) - calculate_rows(start, params.map_size.x);

            for (var i: i32 = 0; i < rows_behind; i+=1){
                let current_row = i / 2;
                var pos: vec2<i32> = vec2<i32> (start.x - i % 2 - current_row, start.y + current_row);
                var vertical_tiles = params.columns;
                if (pos.x < 0 || pos.y < 0) {
                    if (pos.x < pos.y) {
                        vertical_tiles += pos.x;
                        pos.y -= pos.x;
                        pos.x = 0;
                    } else {
                        vertical_tiles += pos.y;
                        pos.x -= pos.y;
                        pos.y = 0;
                    }
                }
                pos.x += vertical_tiles;
                pos.y += vertical_tiles;

                if (pos.x >= params.map_size.x) {
                    let tiles_overflow = pos.x - params.map_size.x;
                    vertical_tiles -= tiles_overflow;
                    pos.y -= tiles_overflow;
                }

                if (pos.y >= params.map_size.y) {
                    let tiles_overflow = pos.y - params.map_size.y;
                    vertical_tiles -= tiles_overflow;
                }
                visible_index += vertical_tiles;
            }

            var columns = get_columns_until_border(index);
            if (actual_row_start.x >= 0 && actual_row_start.y >= 0) {
                columns -= get_columns_until_border(actual_row_start);
            }

            visible_index += columns;
        //calc index for visible tiles array end


    	visble_tiles_cp.tiles[visible_index] = all_tiles.tiles[index.y * params.map_size.x + index.x];
}



//==============================================================================
// Vertex shader
//==============================================================================
struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) tex_coords: vec4<f32>,
    @builtin(instance_index) instance_index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0) var<storage, read> visble_tiles : TileStorage;
@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {

    let tileID = input.instance_index;
    let tile = visble_tiles.tiles[tileID];
    var output : VertexOutput;

    let atlasCoordinate = vec3<u32>(
        tile.AtlasCoord & 0x000000ffu,
        (tile.AtlasCoord & 0x0000ff00u) >> 8u,
        tile.AtlasCoord >> 16u
    );

    let imageSize = vec2<f32>(30.0, 64.0);

    let numberOfTextures = vec2<f32>(2048.0, 2048.0) / imageSize;

    let position = input.position.xy * imageSize - vec2(imageSize.x / 2.0, imageSize.y);

    // Calculate position with camera
    var pos = vec4(position.xy + tile.InstanceTransform.xy, tile.InstanceTransform.z, 1.0);
    pos = camera.view_proj * pos ;

    output.position = pos;
    output.tex_coords = vec2<f32>(
      (input.position.x / numberOfTextures.x) + (1.0 / numberOfTextures.x * f32(atlasCoordinate.x)),
      (input.position.y / numberOfTextures.y) + (1.0 / numberOfTextures.y * f32(atlasCoordinate.y))
    );
    //output.tex_coords = input.tex_coords.xy;
    return output;
}

//==============================================================================
// Fragment shader
//==============================================================================
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    if(color.a <= 0.0){
        discard;
    }
    return color;
}



 

 