#pragma once
/* Shared Use License: This file is owned by Derivative Inc. (Derivative) and
* can only be used, and/or modified for use, in conjunction with
* Derivative's TouchDesigner software, and only if you are a licensee who has
* accepted Derivative's TouchDesigner license or assignment agreement (which
* also govern the use of this file).  You may share a modified version of this
* file with another authorized licensee of Derivative's TouchDesigner software.
* Otherwise, no redistribution or sharing of this file, with or without
* modification, is permitted.
*/

#include "SOP_CPlusPlusBase.h"
#include <string>



// To get more help about these functions, look at SOP_CPlusPlusBase.h
class SimpleShapes : public SOP_CPlusPlusBase
{
public:

	SimpleShapes(const OP_NodeInfo* info);

	virtual ~SimpleShapes();

	virtual void	getGeneralInfo(SOP_GeneralInfo*) override;

	virtual void	execute(SOP_Output*, OP_Inputs*, void* reserved) override;


	virtual void executeVBO(SOP_VBOOutput* output, OP_Inputs* inputs,
							void* reserved) override;


	virtual int32_t getNumInfoCHOPChans() override;

	virtual void getInfoCHOPChan(int index, OP_InfoCHOPChan* chan) override;

	virtual bool getInfoDATSize(OP_InfoDATSize* infoSize) override;

	virtual void getInfoDATEntries(int32_t index, int32_t nEntries,
									OP_InfoDATEntries* entries) override;

	virtual void setupParameters(OP_ParameterManager* manager) override;
	virtual void pulsePressed(const char* name) override;

private:

	// example functions for generating a geometry, change them with any
	// fucntions and algorithm:

	void cubeGeometry(SOP_Output* output, float scale = 1.0f);

	void triangleGeometry(SOP_Output* output);

	void cubeGeometryVBO(SOP_VBOOutput* output, float scale = 1.0f);

	void triangleGeometryVBO(SOP_VBOOutput* output);


	// We don't need to store this pointer, but we do for the example.
	// The OP_NodeInfo class store information about the node that's using
	// this instance of the class (like its name).
	const OP_NodeInfo*		myNodeInfo;

	// In this example this value will be incremented each time the execute()
	// function is called, then passes back to the SOP
	int32_t					myExecuteCount;


	double					myOffset;	
	std::string             myChopChanName;
	float                   myChopChanVal;
	std::string             myChop;

	std::string             myDat;


};
