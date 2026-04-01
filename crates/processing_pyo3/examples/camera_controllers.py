from mewnala import *                                                                                                                    
                                                                                                                                         
angle = 0.0                                                                                                                              
mode = 0                                                                                                                                 
                                                                                                                                         
def setup():                                                                                                                             
    size(800, 600)
    mode_3d()
    orbit_camera()                                                                                                                       

    dir_light = create_directional_light((1.0, 0.98, 0.95), 1500.0)                                                                        
    dir_light.position(300.0, 400.0, 300.0)
    dir_light.look_at(0.0, 0.0, 0.0)                                                                                                     
                
def draw():
    global angle, mode
                                                                                                                                         
    if key_just_pressed(KEY_1):
        mode_3d()                                                                                                                        
        orbit_camera()
        mode = 0
    if key_just_pressed(KEY_2):                                                                                                          
        mode_3d()
        free_camera()                                                                                                                    
        mode = 1
    if key_just_pressed(KEY_3):
        mode_2d()                                                                                                                        
        pan_camera()
        mode = 2                                                                                                                         
                
    background(13, 13, 18)
    if mode < 2:
        fill(255, 217, 145)
        roughness(0.3)                                                                                                                   
        metallic(0.8)
        push_matrix()                                                                                                                    
        rotate(angle)
        box(100.0, 100.0, 100.0)
        pop_matrix()                                                                                                                     
    else:
        fill(204, 77, 51)                                                                                                                
        rect(300.0, 200.0, 200.0, 200.0)                                                                                                 

    angle += 0.02                                                                                                                        
                
run()
