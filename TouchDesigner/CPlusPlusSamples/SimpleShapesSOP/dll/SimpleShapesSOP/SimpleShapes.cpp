/* Shared Use License: This file is owned by Derivative Inc. (Derivative) and
* can only be used, and/or modified for use, in conjunction with
* Derivative's TouchDesigner software, and only if you are a licensee who has
* accepted Derivative's TouchDesigner license or assignment agreement (which
* also govern the use of this file).  You may share a modified version of this
* file with another authorized licensee of Derivative's TouchDesigner software.
* Otherwise, no redistribution or sharing of this file, with or without
* modification, is permitted.
*/

#include "SimpleShapes.h"

#include <stdio.h>
#include <string.h>
#include <math.h>
#include <assert.h>

// These functions are basic C function, which the DLL loader can find
// much easier than finding a C++ Class.
// The DLLEXPORT prefix is needed so the compile exports these functions from the .dll
// you are creating
extern "C"
{

	DLLEXPORT
	int32_t
	GetSOPAPIVersion(void)
	{
		// Always return SOP_CPLUSPLUS_API_VERSION in this function.
		return SOP_CPLUSPLUS_API_VERSION;
	}

	DLLEXPORT
	SOP_CPlusPlusBase*
	CreateSOPInstance(const OP_NodeInfo* info)
	{
		// Return a new instance of your class every time this is called.
		// It will be called once per SOP that is using the .dll
		return new SimpleShapes(info);
	}

	DLLEXPORT
	void
	DestroySOPInstance(SOP_CPlusPlusBase* instance)
	{
		// Delete the instance here, this will be called when
		// Touch is shutting down, when the SOP using that instance is deleted, or
		// if the SOP loads a different DLL
		delete (SimpleShapes*)instance;
	}

};


SimpleShapes::SimpleShapes(const OP_NodeInfo* info) : myNodeInfo(info)
{
	myExecuteCount = 0;
	myOffset = 0.0;
	myChop = "";

	myChopChanName = "";
	myChopChanVal = 0;

	myDat = "N/A";
}

SimpleShapes::~SimpleShapes()
{

}

void
SimpleShapes::getGeneralInfo(SOP_GeneralInfo* ginfo)
{
	// This will cause the node to cook every frame
	ginfo->cookEveryFrameIfAsked = true;

	//if direct to GPU loading:
	ginfo->directToGPU = false;
}


//-----------------------------------------------------------------------------------------------------
//										Generate a geometry on CPU
//-----------------------------------------------------------------------------------------------------

void
SimpleShapes::cubeGeometry(SOP_Output* output, float scale)
{
	// to generate a geometry:
	// addPoint() is the first function to be called.
	// then we can add normals, colors, and any custom attributes for the points
	// last function can be either addParticleSystem() or addTriangle()

	// front
	output->addPoint(-1.0f*scale, -1.0f, 1.0f);
	output->addPoint(1.0f*scale, -1.0f, 1.0f);
	output->addPoint(1.0f*scale, 1.0f, 1.0f);
	output->addPoint(-1.0f*scale, 1.0, 1.0);
	// back
	output->addPoint(-1.0f*scale, -1.0f, -1.0f);
	output->addPoint(1.0f*scale, -1.0f, -1.0f);
	output->addPoint(1.0f*scale, 1.0f, -1.0f);
	output->addPoint(-1.0f*scale, 1.0f, -1.0f);

	float normal[] = {
		// front
		1.0, 0.0, 0.0,
		0.0, 1.0, 0.0,
		0.0, 0.0, 1.0,
		1.0, 1.0, 1.0,
		// back
		1.0, 0.0, 0.0,
		0.0, 1.0, 0.0,
		0.0, 0.0, 1.0,
		1.0, 1.0, 1.0,
	};

	float color[] =
	{
		// front colors
		1.0, 0.0, 0.0, 1.0,
		0.0, 1.0, 0.0, 1.0,
		0.0, 0.0, 1.0, 1.0,
		1.0, 1.0, 1.0, 1.0,
		// back colors
		1.0, 0.0, 0.0, 1.0,
		0.0, 1.0, 0.0, 1.0,
		0.0, 0.0, 1.0, 1.0,
		1.0, 1.0, 1.0, 1.0,
	};

	float color2[] =
	{
		// front colors
		1.0, 0.0, 0.0, 1.0,
		1.0, 0.0, 0.0, 1.0,
		1.0, 0.0, 0.0, 1.0,
		1.0, 0.0, 0.0, 1.0,
		// back colors
		1.0, 0.0, 0.0, 1.0,
		1.0, 0.0, 0.0, 1.0,
		1.0, 0.0, 0.0, 1.0,
		1.0, 0.0, 0.0, 1.0,
	};

	// indices of the input vertices

	int32_t vertices[] = {
		// front
		0, 1, 2,
		2, 3, 0,
		// top
		1, 5, 6,
		6, 2, 1,
		// back
		7, 6, 5,
		5, 4, 7,
		// bottom
		4, 0, 3,
		3, 7, 4,
		// left
		4, 5, 1,
		1, 0, 4,
		// right
		3, 2, 6,
		6, 7, 3,
	};

	int sz = 8;


	for (int32_t i = 0; i < sz; ++i)
	{
		output->setNormal(normal[i * 3 + 0], normal[i * 3 + 1], normal[i * 3 + 2], i);
		output->setColor(color[i * 4 + 0], color[i * 4 + 1], color[i * 4 + 2], color[i * 4 + 3], i);
	}

	output->setCustomAttribute("customColor", 4, AttribType::Float, color2, sz);


	for (int i = 0; i < 12; i++)
	{
		output->addTriangle(vertices[i * 3],
							vertices[i * 3 + 1],
							vertices[i * 3 + 2]);
	}

}

void
SimpleShapes::triangleGeometry(SOP_Output* output)
{
	int32_t vertices[3] = { 0, 1, 2 };

	int sz = 3;

	output->addPoint(0.0f, 0.0f, 0.0f);
	output->addPoint(0.0f, 2.0f, 0.0f);
	output->addPoint(2.0f, 0.0f, 0.0f);

	float normals[3] = { 0.0f, 0.0f, 1.0f };

	output->setNormal(normals[0], normals[1], normals[2], 0);
	output->setNormal(normals[0], normals[1], normals[2], 1);
	output->setNormal(normals[0], normals[1], normals[2], 2);

	output->addTriangle(vertices[0], vertices[1], vertices[2]);

}

void
SimpleShapes::execute(SOP_Output* output, OP_Inputs* inputs, void* reserved)
{
	myExecuteCount++;

	if (inputs->getNumInputs() > 0)
	{

		inputs->enablePar("Reset", 0);	// not used
		inputs->enablePar("Shape", 0);	// not used
		inputs->enablePar("Scale", 0);  // not used

		int ind = 0;

		const OP_SOPInput	*sinput = inputs->getInputSOP(0);

		const Position* ptArr = sinput->getPointPositions();
		const Vector* normals = nullptr;
		const Color* colors = nullptr;
		const TexCoord* textures = nullptr;
		int32_t numTextures = 0;


		if (sinput->hasNormals())
		{
			normals = sinput->getNormals()->normals;
		}

		if (sinput->hasColors())
		{
			colors = sinput->getColors()->colors;
		}

		if (sinput->getTextures()->numTextureLayers)
		{
			textures = sinput->getTextures()->textures;
			numTextures = sinput->getTextures()->numTextureLayers;
		}

		for (int i = 0; i < sinput->getNumPoints(); i++)
		{
			output->addPoint(ptArr[i].x, ptArr[i].y, ptArr[i].z);

			if (normals)
			{
				output->setNormal(normals[i].x,
								  normals[i].y,
								  normals[i].z, i);
			}

			if (colors)
			{
				output->setColor(colors[i].r,
								 colors[i].g,
								 colors[i].b,
								 colors[i].a, i);
			}

			if (textures)
			{
				output->setTexture((float*)(textures + (i * numTextures * 3)), numTextures, i);
			}

		}


		for (int i = 0; i < sinput->getNumCustomAttributes(); i++)
		{
			const CustomAttribInfo* customAttr = sinput->getCustomAttribute(i);

			if (customAttr->attribType == AttribType::Float)
			{
				output->setCustomAttribute((char *)customAttr->name, customAttr->numComponents,
										  customAttr->attribType, (float *)customAttr->floatData, sinput->getNumPoints());
			}
			else
			{
				output->setCustomAttribute((char *)customAttr->name, customAttr->numComponents,
											customAttr->attribType, (int32_t *)customAttr->intData, sinput->getNumPoints());
			}
		}


		for (int i = 0; i < sinput->getNumPrimitives(); i++)
		{

			const PrimitiveInfo primInfo = sinput->getPrimitive(i);

			const int32_t* primVert = primInfo.pointIndices;

			// Note: the addTriangle() assumes that the input SOP has triangulated geometry,
			// if the input geometry is not a triangle, you need to convert it to triangles first:
			output->addTriangle(*(primVert), *(primVert + 1), *(primVert + 2));
		}

	}
	else
	{
		inputs->enablePar("Shape", 1);

		int shape = inputs->getParInt("Shape");

		inputs->enablePar("Scale", 1);
		double	 scale = inputs->getParDouble("Scale");

		// if there is a input chop parameter:
		const OP_CHOPInput	*cinput = inputs->getParCHOP("Chop");
		if (cinput)
		{
			int numSamples = cinput->numSamples;
			int ind = 0;
			myChopChanName = std::string(cinput->getChannelName(0));
			myChop = inputs->getParString("Chop");

			myChopChanVal = float(cinput->getChannelData(0)[ind]);
			scale = float(cinput->getChannelData(0)[ind] * scale);

		}

		switch (shape)
		{
			case 0:		// cube
				cubeGeometry(output, (float)scale);
				break;

			case 1:		// triangle
				triangleGeometry(output);
				break;

			default:
				cubeGeometry(output, (float)scale);
				break;
		}

	}


}

//-----------------------------------------------------------------------------------------------------
//								Generate a geometry and load it straight to GPU (faster)
//-----------------------------------------------------------------------------------------------------

// fillFaceVBO() get the vertices, normals, colors and triangles buffer pointers and then fills in the
// buffers with the input arguments and their sizes.

void
fillFaceVBO(SOP_VBOOutput* output,
	float *InVert, float *InNormal, float *InColor, int32_t  *inIdx, int VertSz, int triSize,
	float scale = 1.0f)
{

	float *vertOut = output->getPos();
	float *normalOut = output->getNormals();
	float *colorOut = output->getColors();
	int32_t *indexBuffer = output->getTriangles(triSize);


	for (int i = 0; i < triSize; i++)
	{
		*(indexBuffer++) = inIdx[i * 3 + 0];
		*(indexBuffer++) = inIdx[i * 3 + 1];
		*(indexBuffer++) = inIdx[i * 3 + 2];
	}

	int k = 0;
	while (k < VertSz * 3)
	{

		*(vertOut++) = InVert[k * 3 + 0] * scale;
		*(vertOut++) = InVert[k * 3 + 1] * scale;
		*(vertOut++) = InVert[k * 3 + 2] * scale;

		if (output->hasNormal())
		{
			*(normalOut++) = InNormal[k * 3 + 0];
			*(normalOut++) = InNormal[k * 3 + 1];
			*(normalOut++) = InNormal[k * 3 + 2];
		}

		if (output->hasColor())
		{
			*(colorOut++) = InColor[k * 4 + 0];
			*(colorOut++) = InColor[k * 4 + 1];
			*(colorOut++) = InColor[k * 4 + 2];
			*(colorOut++) = InColor[k * 4 + 3];
		}
		k++;
	}
}

void
SimpleShapes::cubeGeometryVBO(SOP_VBOOutput* output, float scale)
{
	float pointArr[] =
	{
		//front
		-1.0, -1.0, 1.0, //v0
		1.0, -1.0, 1.0, //v1
		1.0, 1.0, 1.0,  //v2
		-1.0, 1.0, 1.0, //v3

		//right
		1.0, 1.0, 1.0, //v2
		1.0, 1.0, -1.0, //v6
		1.0, -1.0, -1.0,//v5
		1.0, -1.0, 1.0,//v1


		//back
		-1.0, -1.0, -1.0, //v4
		1.0, -1.0, -1.0,  //v5
		1.0, 1.0, -1.0,  //v6
		-1.0, 1.0, -1.0, //v7


		//left
		-1.0, -1.0, -1.0, //v4
		-1.0, -1.0, 1.0,// v0
		-1.0, 1.0, 1.0,//v3
		-1.0, 1.0, -1.0,//v7


		//upper
		1.0, 1.0, 1.0,//v1
		-1.0, 1.0, 1.0,//v3
		-1.0, 1.0, -1.0,//v7
		1.0, 1.0, -1.0,//v6


		//bottom
		-1.0, -1.0, -1.0,//v4
		1.0, -1.0, -1.0,//v5
		1.0, -1.0, 1.0,//v1
		-1.0, -1.0, 1.0//v0
	};

	float normals[] =
	{
		//front
		1.0, 0.0, 0.0, //v0
		0.0, 1.0, 0.0,//v1
		0.0, 0.0, 1.0,//v2
		1.0, 1.0, 1.0,//v3

		//right
		0.0, 0.0, 1.0,//v2
		0.0, 0.0, 1.0, //v6
		0.0, 1.0, 0.0,//v5
		0.0, 1.0, 0.0,//v1


		//back
		1.0, 0.0, 0.0, //v4
		0.0, 1.0, 0.0, //v5
		0.0, 0.0, 1.0,//v6
		1.0, 1.0, 1.0,//v7


		//left
		1.0, 0.0, 0.0, //v4
		1.0, 0.0, 0.0,// v0
		1.0, 1.0, 1.0,//v3
		1.0, 1.0, 1.0,//v7


		//upper
		0.0, 1.0, 0.0,//v1
		1.0, 1.0, 1.0,//v3
		1.0, 1.0, 1.0,//v7
		0.0, 0.0, 1.0,//v6


		//bottom
		1.0, 0.0, 0.0,//v4
		0.0, 1.0, 0.0,//v5
		0.0, 1.0, 0.0,//v1
		1.0, 0.0, 0.0,//v0
	};

	float colors[] = {
		//front
		0, 0, 1, 1,
		1, 0, 1, 1,
		1, 1, 1, 1,
		0, 1, 1, 1,

		//ri
		1, 1, 1, 1,
		1, 1, 0, 1,
		1, 0, 0, 1,
		1, 0, 1, 1,

		//ba
		0, 0, 0, 1,
		1, 0, 0, 1,
		1, 1, 0, 1,
		0, 1, 0, 1,

		//le
		0, 0, 0, 1,
		0, 0, 1, 1,
		0, 1, 1, 1,
		0, 1, 0, 1,

		//up
		1, 1, 1, 1,
		0, 1, 1, 1,
		0, 1, 0, 1,
		1, 1, 0, 1,

		//bo
		0, 0, 0, 1,
		1, 0, 0, 1,
		1, 0, 1, 1,
		0, 0, 1, 1
	};

	int32_t vertices[] =
	{
		0,  1,  2,  0,  2,  3,   //front
		4,  5,  6,  4,  6,  7,   //right
		8,  9,  10, 8,  10, 11,  //back
		12, 13, 14, 12, 14, 15,  //left
		16, 17, 18, 16, 18, 19,  //upper
		20, 21, 22, 20, 22, 23
	};

	// fill in the VBO buffers for this cube:

	fillFaceVBO(output, pointArr, normals, colors, vertices, 8, 12, scale);

	return;
}

void
SimpleShapes::triangleGeometryVBO(SOP_VBOOutput* output)
{

	float normals[] =
	{
		1.0, 0.0, 0.0, //v0
		0.0, 1.0, 0.0, //v1
		0.0, 0.0, 1.0, //v2
	};

	float color[] =
	{
		0, 0, 1, 1,
		1, 0, 1, 1,
		1, 1, 1, 1,
	};

	float pointArr[] =
	{
		0.0f, 0.0f, 0.0f,
		0.0f, 1.0f, 0.0f,
		1.0f, 0.0f, 0.0f,
	};

	int32_t vertices[] = { 0, 1, 2 };

	// fill in the VBO buffers for this triangle:

	fillFaceVBO(output, pointArr, normals, color, vertices, 1, 1);

	return;
}


void
SimpleShapes::executeVBO(SOP_VBOOutput* output,
	OP_Inputs* inputs,
	void* reserved)
{
	myExecuteCount++;

	if (!output)
	{
		return;
	}

	if (inputs->getNumInputs() > 0)
	{
		// if there is a input node SOP node

		inputs->enablePar("Reset", 0);	// not used
		inputs->enablePar("Shape", 0);	// not used
		inputs->enablePar("Scale", 0);  // not used
	}
	else
	{
		inputs->enablePar("Shape", 0);

		inputs->enablePar("Scale", 1); // enable the scale selection
		double	 scale = inputs->getParDouble("Scale");

		// In this sample code an input CHOP node parameter is supported,
		// however it is possible to have DAT or TOP inputs as well

		const OP_CHOPInput	*cinput = inputs->getParCHOP("Chop");
		if (cinput)
		{
			int numSamples = cinput->numSamples;
			int ind = 0;
			myChopChanName = std::string(cinput->getChannelName(0));
			myChop = inputs->getParString("Chop");

			myChopChanVal = float(cinput->getChannelData(0)[ind]);
			scale = float(cinput->getChannelData(0)[ind] * scale);

		}

		// if the geometry have normals or colors, call enable functions:

		output->enableNormal();
		output->enableColor();

		// add custom attributes and access them in the GLSL (shader) code:

		output->addCustomAttribute("customColor", 4, AttribType::Float);
		output->addCustomAttribute("customVert", 1, AttribType::Float);

		// he number of vertices and index buffers must be set before generating any geometries:

		int32_t numVertices = 12;
		int32_t numIndices = 12;

		output->allocVBO(numVertices, numIndices, VBOBufferMode::Static);

		cubeGeometryVBO(output, (float)scale);

		// once the geometry VBO buffers are filled in, call this function as the last function:

		output->updateComplete();

	}

}

//-----------------------------------------------------------------------------------------------------
//								CHOP, DAT, and custom parameters
//-----------------------------------------------------------------------------------------------------

int32_t
SimpleShapes::getNumInfoCHOPChans()
{
	// We return the number of channel we want to output to any Info CHOP
	// connected to the CHOP. In this example we are just going to send 4 channels.
	return 4;
}

void
SimpleShapes::getInfoCHOPChan(int32_t index,
	OP_InfoCHOPChan* chan)
{
	// This function will be called once for each channel we said we'd want to return
	// In this example it'll only be called once.

	if (index == 0)
	{
		chan->name = "executeCount";
		chan->value = (float)myExecuteCount;
	}

	if (index == 1)
	{
		chan->name = "offset";
		chan->value = (float)myOffset;
	}

	if (index == 2)
	{
		chan->name = myChop.c_str();
		chan->value = (float)myOffset;
	}

	if (index == 3)
	{
		chan->name = myChopChanName.c_str();
		chan->value = myChopChanVal;
	}
}

bool
SimpleShapes::getInfoDATSize(OP_InfoDATSize* infoSize)
{
	infoSize->rows = 3;
	infoSize->cols = 3;
	// Setting this to false means we'll be assigning values to the table
	// one row at a time. True means we'll do it one column at a time.
	infoSize->byColumn = false;
	return true;
}

void
SimpleShapes::getInfoDATEntries(int32_t index,
	int32_t nEntries,
	OP_InfoDATEntries* entries)
{
	// It's safe to use static buffers here because Touch will make it's own
	// copies of the strings immediately after this call returns
	// (so the buffers can be reuse for each column/row)
	static char tempBuffer1[4096];
	static char tempBuffer2[4096];

	if (index == 0)
	{
		// Set the value for the first column
#ifdef WIN32
		strcpy_s(tempBuffer1, "executeCount");
#else // macOS
		strlcpy(tempBuffer1, "executeCount", sizeof(tempBuffer1));
#endif
		entries->values[0] = tempBuffer1;

		// Set the value for the second column
#ifdef WIN32
		sprintf_s(tempBuffer2, "%d", myExecuteCount);
#else // macOS
		snprintf(tempBuffer2, sizeof(tempBuffer2), "%d", myExecuteCount);
#endif
		entries->values[1] = tempBuffer2;
	}

	if (index == 1)
	{
		// Set the value for the first column
#ifdef WIN32
		strcpy_s(tempBuffer1, "offset");
#else // macOS
		strlcpy(tempBuffer1, "offset", sizeof(tempBuffer1));
#endif
		entries->values[0] = tempBuffer1;

		// Set the value for the second column
#ifdef WIN32
		sprintf_s(tempBuffer2, "%g", myOffset);
#else // macOS
		snprintf(tempBuffer2, sizeof(tempBuffer2), "%g", myOffset);
#endif
		entries->values[1] = tempBuffer2;
	}

	if (index == 2)
	{
		// Set the value for the first column
#ifdef WIN32
		strcpy_s(tempBuffer1, "DAT input name");
#else // macOS
		strlcpy(tempBuffer1, "offset", sizeof(tempBuffer1));
#endif
		entries->values[0] = tempBuffer1;

		// Set the value for the second column
#ifdef WIN32
		strcpy_s(tempBuffer2, myDat.c_str());
#else // macOS
		snprintf(tempBuffer2, sizeof(tempBuffer2), "%g", myOffset);
#endif
		entries->values[1] = tempBuffer2;
	}
}



void
SimpleShapes::setupParameters(OP_ParameterManager* manager)
{
	// CHOP
	{
		OP_StringParameter	np;

		np.name = "Chop";
		np.label = "CHOP";

		OP_ParAppendResult res = manager->appendCHOP(np);
		assert(res == OP_ParAppendResult::Success);
	}

	// scale
	{
		OP_NumericParameter	np;

		np.name = "Scale";
		np.label = "Scale";
		np.defaultValues[0] = 1.0;
		np.minSliders[0] = -10.0;
		np.maxSliders[0] = 10.0;

		OP_ParAppendResult res = manager->appendFloat(np);
		assert(res == OP_ParAppendResult::Success);
	}

	// shape
	{
		OP_StringParameter	sp;

		sp.name = "Shape";
		sp.label = "Shape";

		sp.defaultValue = "Cube";

		const char *names[] = { "Cube", "Triangle" };
		const char *labels[] = { "Cube", "Triangle" };

		OP_ParAppendResult res = manager->appendMenu(sp, 2, names, labels);
		assert(res == OP_ParAppendResult::Success);
	}

	// pulse
	{
		OP_NumericParameter	np;

		np.name = "Reset";
		np.label = "Reset";

		OP_ParAppendResult res = manager->appendPulse(np);
		assert(res == OP_ParAppendResult::Success);
	}

}

void
SimpleShapes::pulsePressed(const char* name)
{
	if (!strcmp(name, "Reset"))
	{
		myOffset = 0.0;
	}
}

